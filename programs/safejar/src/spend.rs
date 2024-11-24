use std::cmp::Ordering;

use anchor_lang;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;
use anchor_lang::solana_program::hash::hash;
use anchor_spl::token::spl_token::native_mint::ID as sol_mint;
use anchor_spl::token::{self, SyncNative, Transfer as TokenTransfer};

use crate::delegate::Delegation;
use crate::errors::TreasuryError;

use crate::rule::{Rule, RuleAccumulator, ZERO_HASH};
use crate::{
    nplog, tree, CompleteSpendRequestDirect, CreateSpendRequestDirect, PROGRAM_DELEGATION_SEED,
};

impl<'info> CreateSpendRequestDirect<'info> {
    pub fn process(&mut self, amount: u64, tree: Vec<u8>) -> ProgramResult {
        if self.delegation_vault.mint == sol_mint {
            token::sync_native(CpiContext::new(
                self.token_program.to_account_info(),
                SyncNative {
                    account: self.destination_vault.to_account_info(),
                },
            ))?;
            token::sync_native(CpiContext::new(
                self.token_program.to_account_info(),
                SyncNative {
                    account: self.delegation_vault.to_account_info(),
                },
            ))?;
        }
        nplog!("np create - 1");
        let context = TransferContext::new(
            &self.destination_vault.mint,
            &self.destination_owner.owner,
            &self.linker.key(),
            &self.delegation_vault.key(),
            &self.destination_vault.key(),
            &amount,
            &self.clock.slot,
        );
        nplog!("create - 2");
        self.request.init(
            &self.delegation.key(),
            &self.delegation.state,
            &context,
            &tree,
        )?;
        nplog!("create - 3");

        Ok(())
    }
}

impl<'info> CompleteSpendRequestDirect<'info> {
    pub fn process(&mut self) -> ProgramResult {
        nplog!("complete - 1");
        self.request.eval()?;
        nplog!("complete - 2");
        // do token spend
        // amount
        let transfer_instruction = TokenTransfer {
            from: self.delegation_vault.to_account_info(),
            to: self.destination_vault.to_account_info(),
            authority: self.delegation.to_account_info(),
        };
        let bump_vector = self.delegation.bump.to_le_bytes();
        // PROGRAM_HOLDING_SEED,controller.key().as_ref(),vault.key().as_ref()
        let controller_id = self.delegation.controller;
        //let vault_id = self.vault.key();
        let inner = vec![
            PROGRAM_DELEGATION_SEED,
            controller_id.as_ref(),
            self.delegation.rule_set_hash.as_ref(),
            bump_vector.as_ref(),
        ];
        let outer = vec![inner.as_slice()];
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            transfer_instruction,
            &outer,
        );
        token::transfer(cpi_ctx, self.request.context.amount)?;

        self.delegation.state.update(&self.request.context)?;
        Ok(())
    }
}

#[account]
pub struct SpendRequest {
    pub delegation: Pubkey,
    pub state: SpendState,
    pub context: TransferContext,
    pub result: u64, // bitwise operation for max of 64 leaf nodes
    pub index: u8,
    pub count: u8,
    pub tree: Vec<u8>, // max size is 300B
    pub hash: [u8; 32],
}

pub const TREE_MAX_SIZE: usize = 300;

impl SpendRequest {
    pub fn init(
        &mut self,
        delegation: &Pubkey,
        state: &SpendState,
        context: &TransferContext,
        tree: &Vec<u8>,
    ) -> Result<()> {
        msg!("s - 1");
        self.delegation = delegation.clone();
        self.state = state.clone();
        //self.state.index+=1;
        let space = self.state.find(&context.mint)?;
        msg!("s - 2");
        space.mint = context.mint;
        space.index += 1;
        // we check if the count has been incremented in the Delegation account in the final instruction
        self.index = 0;
        let (_x, c) = tree::deserialize(tree)?;
        msg!("s - 3");

        self.count = c;
        self.tree = tree.clone();
        self.hash = RuleAccumulator::hash_init();
        self.hash_tree(&tree);
        msg!("s - 4");
        self.context = context.clone();
        msg!("s - 5");
        Ok(())
    }

    // identical to RuleAccumulator
    pub fn hash_tree(&mut self, tree: &Vec<u8>) {
        let mut a = Vec::new();
        a.append(self.hash.to_vec().as_mut());
        a.append(tree.clone().as_mut());
        self.hash = hash(&a).to_bytes();
        //msg!("i=-1 hash={:X?}",&self.hash);
    }

    pub fn process(&mut self, rule: &dyn Rule) -> Result<()> {
        nplog!("sr - 1");
        self.hash = rule.hash(self.index, &self.hash)?;
        nplog!("sr - 2");
        if self.count <= self.index {
            ////msg!("sr - 3");
            return Err(TreasuryError::RuleOutOfRange.into());
        }
        nplog!("sr - 4");
        if rule.process(&mut self.state, &self.context).is_ok() {
            nplog!(
                "sr - 5 - index {} setting result {}",
                self.index,
                1 << self.index
            );
            self.result |= 1 << self.index;
        }
        nplog!("sr - 6: {:064b}", self.result);
        self.index += 1;

        Ok(())
    }

    pub fn eval(&self) -> Result<()> {
        nplog!("eval - 1");
        let mut index: u8 = 0;
        let (root, _c) = tree::deserialize(&self.tree)?;
        nplog!("eval - 2");
        if !tree::evaluate(&mut index, root, &self.result) {
            return Err(TreasuryError::RuleEvalFalse.into());
        }
        nplog!("eval - 3");
        Ok(())
    }
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct SpendState {
    pub list: Vec<SpendStateSlot>,
}

impl SpendState {
    /// Call this when a spend request is being completed..
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn update(&mut self, txctx: &TransferContext) -> Result<()> {
        // find open slot into which we shall update the spend state
        let y = self.find(&txctx.mint)?;

        if !txctx.is_sweep {
            // not a sweep; so we update the spend state for this particular mint
            y.last_slot = txctx.slot;
            y.last_spend = txctx.amount;
            y.generic_score = 0;
            nplog!("record rate limit: {} {}", y.last_slot, y.last_spend);
        } else {
            nplog!("why do we have a sweep?");
        }

        Ok(())
    }
}

pub(crate) fn delegation_account_size(max_spend_state: u8) -> usize {
    return 8
        + std::mem::size_of::<Delegation>()
        + (max_spend_state as usize) * std::mem::size_of::<SpendStateSlot>();
}

impl SpendState {
    pub fn new(size: u8) -> Self {
        let mut list = Vec::new();
        for _i in 0..size {
            list.push(SpendStateSlot::new())
        }
        Self { list }
    }

    pub fn clean(&mut self, cut_off_slot: u64) {
        let mut iterator = self.list.iter_mut();
        while let Some(space) = iterator.next() {
            if space.last_slot < cut_off_slot {
                *space = SpendStateSlot::new()
            }
        }
    }

    /// Find a spend space to store spending history.  The look up is by mint.
    /// Once a space is found, assign a mint.
    ///
    ///
    /// # Errors
    ///
    /// This function will return an error if no empty space is found nor a matching mint.
    pub fn find<'a>(&'a mut self, mint: &'a Pubkey) -> Result<&'a mut SpendStateSlot> {
        nplog!("search for mint {}", mint);
        let n = &self.list.len();
        nplog!("f - 1 - spend state mint {} count {}", mint, n);
        let mut iterator = self.list.iter_mut();

        // look for a matching mint space;
        // and also find an empty space;
        let mut empty = None;
        while let Some(space) = iterator.next() {
            if space.is_blank() {
                if empty.is_none() {
                    empty = Some(space);
                }
            } else {
                nplog!(
                    "f - 2 - space index={} mint {} {}",
                    space.index,
                    mint,
                    space.mint
                );
                if space.mint == *mint {
                    nplog!("f - 3 - matching mint {}", mint);
                    return Ok(space);
                }
            }
        }

        if empty.is_some() {
            let x = empty.unwrap();
            x.mint = mint.clone();
            return Ok(x);
        }
        nplog!("f - 6");
        return Err(TreasuryError::SpendRequestNoSpace.into());
    }
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct SpendStateSlot {
    pub mint: Pubkey,
    pub index: u64,
    pub last_spend: u64,
    pub last_slot: u64,
    pub generic_score: u8,
}

impl SpendStateSlot {
    pub fn new() -> Self {
        Self {
            mint: Pubkey::new_from_array(ZERO_HASH),
            index: 0,
            last_spend: 0,
            last_slot: 0,
            generic_score: u8::MAX - 1,
        }
    }
    pub fn is_blank(&self) -> bool {
        return self.mint == Pubkey::new_from_array(ZERO_HASH);
    }
    pub fn cmp(&self, b: &SpendStateSlot) -> Ordering {
        return self.last_slot.cmp(&b.last_slot);
    }
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct TransferContext {
    pub program_id: Pubkey,
    pub linker: Pubkey, // link rule applications together
    pub source_vault: Pubkey,
    pub destination_vault: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub slot: u64,
    pub is_sweep: bool,
}

impl TransferContext {
    pub fn new(
        mint: &Pubkey,
        program_id: &Pubkey,
        authorizer: &Pubkey,
        source_vault: &Pubkey,
        destination_vault: &Pubkey,
        amount: &u64,
        slot: &u64,
    ) -> Self {
        Self {
            is_sweep: false,
            mint: mint.clone(),
            program_id: program_id.clone(),
            linker: authorizer.clone(),
            source_vault: source_vault.clone(),
            destination_vault: destination_vault.clone(),
            amount: amount.clone(),
            slot: slot.clone(),
        }
    }
}
