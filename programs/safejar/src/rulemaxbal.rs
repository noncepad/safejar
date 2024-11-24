
use anchor_lang::solana_program::entrypoint::ProgramResult;
use anchor_lang::solana_program::hash::HASH_BYTES;
use anchor_lang::prelude::*;

use crate::{RuleAddBalanceConstraint, SpendProcessBalanceConstraint};
use crate::errors::TreasuryError;
use crate::rule::{Rule, generic_hash, RULE_BALANCE_CONSTRAINT};
use crate::spend::{SpendState, TransferContext};


impl<'info> RuleAddBalanceConstraint<'info>{
    pub fn process(&mut self,max_bal: u64)->ProgramResult{
        let rule = BalanceConstraint::new(
            &self.mint.key(),
            0,
            max_bal,
        );
        if self.accumulator.add(&rule).is_err(){
            return Err(ProgramError::Custom(TreasuryError::RuleAddFail.into()))
        }
        Ok(())
    }
}



impl<'info> SpendProcessBalanceConstraint<'info>{
    pub fn process(&mut self,max_bal: u64)->ProgramResult{
        let rule = BalanceConstraint::new(
            &self.delegation_vault.mint,
            self.delegation_vault.amount,
            max_bal,
        );
        self.request.process(&rule)?;
        
        Ok(())
    }
}



#[derive(AnchorDeserialize, AnchorSerialize,Clone)]
pub struct BalanceConstraint{
    pub mint: Pubkey,
    pub balance: u64,
    pub max_bal: u64,
}

impl BalanceConstraint{
    pub fn new(mint: &Pubkey,balance: u64, max_bal: u64)->Self{
        Self{
            mint: mint.clone(),
            balance,
            max_bal,
        }
    }

    pub fn for_serialization(&self)->BalanceConstraintOnly{
        BalanceConstraintOnly{
            mint: self.mint.clone(),
            max_bal: self.max_bal,
        }
    }
}

#[derive(AnchorDeserialize, AnchorSerialize,Clone)]
pub struct BalanceConstraintOnly{
    pub mint: Pubkey,
    pub max_bal: u64,
}


impl<'b> Rule<'b> for BalanceConstraint{
    
    fn id(&self)->u8{
        return RULE_BALANCE_CONSTRAINT
    }
    

    fn process<'a>(
        &'a self,
        _state: &mut SpendState, 
        _context: &TransferContext,
    )->Result<()> {
        // we get the mint from delegation_vault!  max sure it matches the mint in the rule.
        if self.max_bal<self.balance{
            return Err(TreasuryError::RuleMaxBalanceExceeded.into())
        }
        Ok(())
    }

    fn hash<'a>(&'a self,index: u8,prev_hash: &'a[u8])->Result<[u8;HASH_BYTES]> {
        let mut x=[0u8;std::mem::size_of::<BalanceConstraintOnly>()];
        let mut cursor = std::io::Cursor::new(x.as_mut());
        self.for_serialization().serialize(&mut cursor)?;
        //msg!("_______+++++rule({})={:X?}",x.len(),&x);
        return Ok(generic_hash(&index,&x,prev_hash))
    }

  
}

