
use anchor_lang::solana_program::entrypoint::ProgramResult;
use anchor_lang::solana_program::hash::HASH_BYTES;
use anchor_lang::prelude::*;

use crate::{RuleAddProgramConstraint, SpendProcessProgramConstraint};
use crate::errors::TreasuryError;
use crate::rule::{RULE_PROGRAM_CONSTRAINT, Rule, generic_hash};
use crate::spend::{SpendState, TransferContext};


impl<'info> RuleAddProgramConstraint<'info>{
    pub fn process(&mut self)->ProgramResult{
        let rule = ProgramConstraint::new(&self.program.key());
        if self.accumulator.add(&rule).is_err(){
            return Err(ProgramError::Custom(TreasuryError::RuleAddFail.into()))
        }
        Ok(())
    }
}



impl<'info> SpendProcessProgramConstraint<'info>{
    pub fn process(&mut self)->ProgramResult{
        let rule = ProgramConstraint::new(
            &self.required_program.key(),
        );
        self.request.process(&rule)?;
        
        Ok(())
    }
}



#[derive(AnchorDeserialize, AnchorSerialize,Clone)]
pub struct ProgramConstraint{
    pub program_id: Pubkey,

}

impl ProgramConstraint{
    pub fn new(program_id: &Pubkey)->Self{
        Self{
            program_id: program_id.clone(),
        }
    }
}


impl<'b> Rule<'b> for ProgramConstraint{
    
    fn id(&self)->u8{
        return RULE_PROGRAM_CONSTRAINT
    }
    

    fn process<'a>(
        &'a self,
        _state: &mut SpendState, 
        context: &TransferContext,
    )->Result<()> {
        if context.program_id!=self.program_id{
            return Err(TreasuryError::RuleProgramConstraintMustMatch.into())
        }
        Ok(())
    }

    fn hash<'a>(&'a self,index: u8,prev_hash: &'a[u8])->Result<[u8;HASH_BYTES]> {
        let mut x=[0u8;std::mem::size_of::<ProgramConstraint>()];
        let mut cursor = std::io::Cursor::new(x.as_mut());
        self.serialize(&mut cursor)?;
        //msg!("_______+++++rule({})={:X?}",x.len(),&x);
        return Ok(generic_hash(&index,&x,prev_hash))
    }

  
}

