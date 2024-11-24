use crate::errors::TreasuryError;
use crate::spend::SpendState;
use crate::{
    nplog, ApproveDelegation, CloseDelegation, Delegate, RejectDelegation, ID,
    PROGRAM_DELEGATION_SEED,
};
use anchor_lang;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;
use anchor_lang::solana_program::hash::HASH_BYTES;

#[account]
pub struct Delegation {
    pub bump: u8,
    pub controller: Pubkey,
    pub rule_set_count: u8,
    pub rule_set_hash: [u8; 32],
    pub state: SpendState,
    pub requested_slot: u64,
}

// controller at offset=8+1
// rule_set_hash at offset=8+1+32+8

impl<'a> Delegation {
    pub fn init(
        &mut self,
        controller: &'a Pubkey,
        bump: u8,
        rule_set_count: u8,
        rule_set_hash: &[u8],
        slot: u64,
        max_spend_state: u8,
    ) -> Result<()> {
        nplog!("delegate - 1");

        if rule_set_hash.len() != HASH_BYTES {
            nplog!("delegate - 1a");
            return Err(TreasuryError::BadHash.into());
        }
        nplog!("delegate - 2");
        for i in 0..HASH_BYTES {
            nplog!("delegate - 2a - i={}", i);
            self.rule_set_hash[i] = rule_set_hash[i];
        }
        self.bump = bump;
        self.controller = controller.clone();
        self.rule_set_count = rule_set_count;
        nplog!("setting spend state max to {}", max_spend_state);
        self.state = SpendState::new(max_spend_state);
        self.requested_slot = slot;
        nplog!("delegate - 3");
        Ok(())
    }
}

impl<'info> Delegate<'info> {
    pub fn process(&mut self, bump: u8, max_spend_state: u8) -> ProgramResult {
        nplog!("delegate - 1");
        self.controller.delegation_count += 1;
        nplog!("delegate - 2");

        self.delegation.init(
            &self.controller.key(),
            bump, // the bump is no longer used
            self.accumulator.count,
            &self.accumulator.hash,
            self.clock.slot,
            max_spend_state,
        )?;

        nplog!("delegate - 3");
        Ok(())
    }
}

impl<'info> ApproveDelegation<'info> {
    pub fn process(&mut self) -> ProgramResult {
        self.delegation.requested_slot = 0;

        Ok(())
    }
}

impl<'info> RejectDelegation<'info> {
    pub fn process(&mut self) -> ProgramResult {
        msg!("reject - 1");

        Ok(())
    }
}

impl<'info> CloseDelegation<'info> {
    pub fn process(&mut self) -> ProgramResult {
        self.controller.delegation_count -= 1;
        Ok(())
    }
}

pub fn delegation_id(controller: &Pubkey, hash: &[u8; 32]) -> Pubkey {
    let x = [PROGRAM_DELEGATION_SEED, controller.as_ref(), hash.as_ref()];
    let (ans, _bump) = Pubkey::find_program_address(&x, &ID);
    return ans;
}
