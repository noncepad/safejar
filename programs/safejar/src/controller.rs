use crate::{
    nplog, CloseController, CloseControllerVault, CreateController, TransferToController,
    TransferToDelegation, ID, PROGRAM_CONTROLLER_SEED, PROGRAM_DELEGATION_SEED,
};
use anchor_lang;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;
use anchor_spl::token::{self, CloseAccount, SyncNative, Transfer as TokenTransfer};
use token::spl_token::native_mint::ID as sol_mint;

#[account]
pub struct Controller {
    pub bump: u8,
    pub owner: Pubkey, // link to Mint account for equity token
    pub rules: Pubkey,
    pub delegation_count: u32,
}

impl Controller {
    pub fn init(&mut self, bump: &u8, owner: &Pubkey) {
        self.bump = bump.clone();
        self.owner = owner.clone();
        self.delegation_count = 0;
    }
}

impl<'info> CreateController<'info> {
    pub fn process(&mut self, bump: &u8) -> ProgramResult {
        nplog!("++hello noncepad 0");
        msg!("hello __0");
        self.controller.init(bump, &self.owner.key());
        nplog!("hello noncepad 123");

        return Ok(());
    }
}

impl<'info> CloseControllerVault<'info> {
    pub fn process(&mut self) -> ProgramResult {
        let close_instruction = CloseAccount {
            account: self.controller_vault.to_account_info(),
            destination: self.owner_vault.to_account_info(),
            authority: self.controller.to_account_info(),
        };

        let bump_vector = self.controller.bump.to_le_bytes();
        let inner = vec![
            PROGRAM_CONTROLLER_SEED,
            self.controller.owner.as_ref(),
            bump_vector.as_ref(),
        ];
        let outer = vec![inner.as_slice()];
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            close_instruction,
            &outer,
        );
        token::close_account(cpi_ctx)?;
        return Ok(());
    }
}

impl<'info> TransferToController<'info> {
    pub fn process(&mut self, amount: u64) -> ProgramResult {
        //msg!("delete me later - 1");
        //msg!("transfering amount={}",amount);

        //msg!("to controller");
        let close_token_account = amount == 0 || self.delegation_vault.amount == amount;
        if self.controller_vault.mint == sol_mint {
            token::sync_native(CpiContext::new(
                self.token_program.to_account_info(),
                SyncNative {
                    account: self.controller_vault.to_account_info(),
                },
            ))?;
            token::sync_native(CpiContext::new(
                self.token_program.to_account_info(),
                SyncNative {
                    account: self.delegation_vault.to_account_info(),
                },
            ))?;
        }
        {
            // always do this section; i use an if block to isolate variables in this scope
            let cpi_ctx;
            let transfer_amount;
            if amount == 0 {
                transfer_amount = self.delegation_vault.amount;
            } else {
                transfer_amount = amount;
            }
            let transfer_instruction = TokenTransfer {
                from: self.delegation_vault.to_account_info(),
                to: self.controller_vault.to_account_info(),
                authority: self.delegation.to_account_info(),
            };
            let bump_vector = self.delegation.bump.to_le_bytes();
            // PROGRAM_HOLDING_SEED,controller.key().as_ref(),vault.key().as_ref()
            let controller_id = self.controller.key();
            //let vault_id = self.delegation_vault.key();
            let inner = vec![
                PROGRAM_DELEGATION_SEED,
                controller_id.as_ref(),
                self.delegation.rule_set_hash.as_ref(),
                bump_vector.as_ref(),
            ];
            let outer = vec![inner.as_slice()];
            cpi_ctx = CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                transfer_instruction,
                &outer,
            );
            token::transfer(cpi_ctx, transfer_amount)?;
        }

        // close token vault
        if close_token_account {
            let cpi_ctx;
            // sweeping
            let close_instruction = CloseAccount {
                account: self.delegation_vault.to_account_info(),
                destination: self.controller_vault.to_account_info(),
                authority: self.delegation.to_account_info(),
            };
            let controller_id = self.controller.key();
            let bump_vector = self.delegation.bump.to_le_bytes();
            let inner = vec![
                PROGRAM_DELEGATION_SEED,
                controller_id.as_ref(),
                self.delegation.rule_set_hash.as_ref(),
                bump_vector.as_ref(),
            ];
            let outer = vec![inner.as_slice()];
            cpi_ctx = CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                close_instruction,
                &outer,
            );
            token::close_account(cpi_ctx)?;
        }

        Ok(())
    }
}

impl<'info> TransferToDelegation<'info> {
    pub fn process(&mut self, amount: u64) -> ProgramResult {
        if self.controller_vault.mint == sol_mint {
            token::sync_native(CpiContext::new(
                self.token_program.to_account_info(),
                SyncNative {
                    account: self.controller_vault.to_account_info(),
                },
            ))?;
            token::sync_native(CpiContext::new(
                self.token_program.to_account_info(),
                SyncNative {
                    account: self.delegation_vault.to_account_info(),
                },
            ))?;
        }
        let close_token_account = amount == 0 || self.delegation_vault.amount == amount;
        {
            //msg!("transfering amount={}",amount);
            let transfer_amount;
            let cpi_ctx_transfer;

            //msg!("to delegation");
            transfer_amount = amount;
            let transfer_instruction = TokenTransfer {
                from: self.controller_vault.to_account_info(),
                to: self.delegation_vault.to_account_info(),
                authority: self.controller.to_account_info(),
            };
            let bump_vector = self.controller.bump.to_le_bytes();
            // PROGRAM_HOLDING_SEED,controller.key().as_ref(),vault.key().as_ref()
            let owner_id = self.controller.owner;

            let inner = vec![
                PROGRAM_CONTROLLER_SEED,
                owner_id.as_ref(),
                bump_vector.as_ref(),
            ];
            let outer = vec![inner.as_slice()];
            cpi_ctx_transfer = CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                transfer_instruction,
                &outer,
            );
            token::transfer(cpi_ctx_transfer, transfer_amount)?;
        }

        // close token vault
        if close_token_account {
            let cpi_ctx;
            // sweeping
            let close_instruction = CloseAccount {
                account: self.controller_vault.to_account_info(),
                destination: self.delegation_vault.to_account_info(),
                authority: self.controller.to_account_info(),
            };
            let bump_vector = self.controller.bump.to_le_bytes();
            // PROGRAM_HOLDING_SEED,controller.key().as_ref(),vault.key().as_ref()
            let owner_id = self.controller.owner;

            let inner = vec![
                PROGRAM_CONTROLLER_SEED,
                owner_id.as_ref(),
                bump_vector.as_ref(),
            ];
            let outer = vec![inner.as_slice()];
            cpi_ctx = CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                close_instruction,
                &outer,
            );
            token::close_account(cpi_ctx)?;
        }

        Ok(())
    }
}

impl<'info> CloseController<'info> {
    pub fn process(&mut self) -> ProgramResult {
        return Ok(());
    }
}

pub fn controller_id(owner: &Pubkey) -> Pubkey {
    let x = [PROGRAM_CONTROLLER_SEED, owner.as_ref()];
    let (ans, _bump) = Pubkey::find_program_address(&x, &ID);
    return ans;
}
