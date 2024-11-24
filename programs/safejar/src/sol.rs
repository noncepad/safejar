


use anchor_lang::{
    prelude::*, 
    solana_program::{
        entrypoint::ProgramResult,
        program::invoke,
        system_instruction,
    },
};
use anchor_spl::token::{
    self, 
    SyncNative, CloseAccount,
};

use anchor_lang;
use crate::{RentManagerTransferSOL, RentManagerSubsidize};


impl<'info> RentManagerTransferSOL<'info>{
    pub fn process(&mut self)->ProgramResult{
        //msg!("delete me later - 1");
        //msg!("owner is signer? {}",self.source_owner.is_signer);
        token::close_account(
            CpiContext::new(
                self.token_program.to_account_info(),
                CloseAccount{
                    account: self.source_vault.to_account_info(),
                    destination: self.tmp_sys.to_account_info(),
                    authority: self.source_owner.to_account_info(),
                }
            )
        )?;
        return Ok(())
    }
}



impl<'info> RentManagerSubsidize<'info>{
    pub fn process(&mut self,amount: u64)->ProgramResult{
        let remainder;
        match self.tmp_sys.to_account_info().lamports().checked_sub(amount){
            Some(x) => {
                remainder = x;
            },
            None => {
                return Err(ProgramError::InsufficientFunds)
            },
        }
        // send "amount" to "target"
        //msg!("transfer - 1");
        invoke(
            &system_instruction::transfer(
                &self.tmp_sys.key(),
                &self.target.key(),
                amount,
            ),
            &[
                self.tmp_sys.to_account_info(),
                self.target.to_account_info(),
            ],
        )?;
        // send change to "change_sol"
        //msg!("transfer - 2");
        invoke(
            &system_instruction::transfer(
                &self.tmp_sys.key(),
                &self.change_sol.key(),
                remainder,
            ),
            &[
                self.tmp_sys.to_account_info(),
                self.change_sol.to_account_info(),
            ],
        )?;
        //msg!("transfer - 3");
        // sync change_sol so lamports match token balance
        token::sync_native(
            CpiContext::new(
                self.token_program.to_account_info(),
                SyncNative{
                    account: self.change_sol.to_account_info(),
                }
            )
        )?;
        //msg!("transfer - 4");

        // the tmp_sys balance is zero, so the account will be deleted
        return Ok(())
    }
}

