use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;
use anchor_spl::token::{Transfer as TokenTransfer, self, CloseAccount, SyncNative};


use crate::{ConsolidateVault, PROGRAM_DELEGATION_SEED};

impl<'info> ConsolidateVault<'info>{
    fn transfer(
        source_ai: &AccountInfo<'info>,
        source_balance: u64,
        outer: &Vec<&[&[u8]]>,
        destination_ai: &AccountInfo<'info>,
        delegation_ai: &AccountInfo<'info>,
        rent_sol_vault_ai: &AccountInfo<'info>,
        token_program_ai: &AccountInfo<'info>,
    )->Result<()>{
        // transfer all of the tokens out and put them in the ATA token account.
        // move the remaining SOL to the rent_sol_vault account, which is also controlled by the delegation account
        
        let transfer_instruction = TokenTransfer{
            from: source_ai.clone(),
            to:  destination_ai.clone(),
            authority: delegation_ai.clone(),
        };

        let cpi_ctx=CpiContext::new_with_signer(
        token_program_ai.clone(),
        transfer_instruction,
        &outer,
        );
        token::transfer(cpi_ctx, source_balance)?;

        let close_instruction = CloseAccount{
            account: source_ai.clone(),
            destination: rent_sol_vault_ai.clone(),
            authority: delegation_ai.clone(),
        };
        
        token::close_account(
            CpiContext::new_with_signer(
            token_program_ai.clone(),
            close_instruction,
            &outer,
            ),
        )?;
        return Ok(())
    }
    pub fn process(&mut self)->ProgramResult{
        let delegation_ai = self.delegation.to_account_info();
        let destination_ai=self.ata_vault.to_account_info();
        let bump_vector = self.delegation.bump.to_le_bytes();
        let controller_id = self.delegation.controller;
        //let vault_id = self.vault.key();
        let inner = Box::new(vec![
        PROGRAM_DELEGATION_SEED,
        controller_id.as_ref(),
        self.delegation.rule_set_hash.as_ref(),
        bump_vector.as_ref(),
        ]);
        let outer = vec![inner.as_slice()];
        
        match Some(&self.vault_1){
            Some(source) => {
                ConsolidateVault::transfer(
                    &source.to_account_info(),
                    source.amount,
                    &outer,
                    &destination_ai,
                    &delegation_ai,
                    &self.rent_sol_vault.to_account_info(),
                    &self.token_program.to_account_info(),
                )?;
            },
            None => {},
        }
        match &self.vault_2{
            Some(source) => {
                ConsolidateVault::transfer(
                    &source.to_account_info(),
                    source.amount,
                    &outer,
                    &destination_ai,
                    &delegation_ai,
                    &self.rent_sol_vault.to_account_info(),
                    &self.token_program.to_account_info(),
                )?;
            },
            None => {},
        }
        match &self.vault_3{
            Some(source) => {
                ConsolidateVault::transfer(
                    &source.to_account_info(),
                    source.amount,
                    &outer,
                    &destination_ai,
                    &delegation_ai,
                    &self.rent_sol_vault.to_account_info(),
                    &self.token_program.to_account_info(),
                )?;
            },
            None => {},
        }
        match &self.vault_4{
            Some(source) => {
                ConsolidateVault::transfer(
                    &source.to_account_info(),
                    source.amount,
                    &outer,
                    &destination_ai,
                    &delegation_ai,
                    &self.rent_sol_vault.to_account_info(),
                    &self.token_program.to_account_info(),
                )?;
            },
            None => {},
        }
        match &self.vault_5{
            Some(source) => {
                ConsolidateVault::transfer(
                    &source.to_account_info(),
                    source.amount,
                    &outer,
                    &destination_ai,
                    &delegation_ai,
                    &self.rent_sol_vault.to_account_info(),
                    &self.token_program.to_account_info(),
                )?;
            },
            None => {},
        }
        
        token::sync_native(
            CpiContext::new(
                self.token_program.to_account_info(),
                SyncNative{
                    account: self.rent_sol_vault.to_account_info(),
                },
            )
        )?;
        
        
        return Ok(())
    }
}