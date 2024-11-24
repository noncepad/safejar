
use anchor_lang::solana_program::entrypoint::ProgramResult;
use anchor_lang::solana_program::hash::HASH_BYTES;
use anchor_lang::prelude::*;

use crate::{RuleAddSweep, RuleAddSweepATA, SpendProcessSweep};
use crate::errors::TreasuryError;
use crate::rule::{Rule, generic_hash, RULE_SWEEP};
use crate::spend::{SpendState, TransferContext};


impl<'info> RuleAddSweep<'info>{
    pub fn process(&mut self, min_bal: u64)->ProgramResult{
        let rule = Sweep::new(
            &self.destination.key(),
            &self.destination.mint,
            self.destination.amount,
            min_bal,
        );
        if self.accumulator.add(&rule).is_err(){
            return Err(ProgramError::Custom(TreasuryError::RuleAddFail.into()))
        }
        Ok(())
    }
}


impl<'info> RuleAddSweepATA<'info>{
    pub fn process(&mut self, min_bal: u64)->ProgramResult{
        let rule = Sweep::new(
            &self.destination.key(),
            &self.destination.mint,
            self.destination.amount,
            min_bal,
        );
        if self.accumulator.add(&rule).is_err(){
            return Err(ProgramError::Custom(TreasuryError::RuleAddFail.into()))
        }
        Ok(())
    }
}


impl<'info> SpendProcessSweep<'info>{
    pub fn process(&mut self,min_bal: u64)->ProgramResult{
        let rule = Sweep::new(
            &self.required_destination.key(),
            &self.required_destination.mint,
            self.required_destination.amount,
            min_bal,
        );
        self.request.process(&rule)?;

        // we mark this spend request as a sweep so that the rate limit is not incremented
        self.request.context.is_sweep=true;
        
        Ok(())
    }
}



#[derive(AnchorDeserialize, AnchorSerialize,Clone)]
pub struct Sweep{
    pub destination: Pubkey,
    pub mint: Pubkey,
    pub balance: u64,
    pub min_bal: u64,
}

#[derive(AnchorDeserialize, AnchorSerialize,Clone)]
pub struct SweepOnly{
    pub destination: Pubkey,
    pub min_bal: u64,
}

impl Sweep{
    pub fn new(destination: &Pubkey, mint: &Pubkey,balance: u64, min_bal: u64)->Self{
        Self{
            destination: destination.clone(),
            balance,
            mint: mint.clone(),
            min_bal,
        }
    }
    pub fn for_serialization(&self)->SweepOnly{
        SweepOnly { destination: self.destination.clone(), min_bal: self.min_bal }
    }
}


impl<'b> Rule<'b> for Sweep{
    
    fn id(&self)->u8{
        return RULE_SWEEP
    }
    

    fn process<'a>(
        &'a self,
        _state: &mut SpendState, 
        context: &TransferContext,
    )->Result<()> {
        if context.destination_vault!=self.destination{
            return Err(TreasuryError::RuleSweepWrongDestination.into())
        }
        if self.balance<self.min_bal{
            return Err(TreasuryError::RuleSweepNotEnoughFunds.into())
        }
        Ok(())
    }

    fn hash<'a>(&'a self,index: u8,prev_hash: &'a[u8])->Result<[u8;HASH_BYTES]> {
        
        let mut x=[0u8;std::mem::size_of::<SweepOnly>()];
        let so =self.for_serialization();
        let mut cursor = std::io::Cursor::new(x.as_mut());
        so.serialize(&mut cursor)?;
        //msg!("_______+++++rule({})={:X?}",x.len(),&x);
        return Ok(generic_hash(&index,&x,prev_hash))
    }

  
}

