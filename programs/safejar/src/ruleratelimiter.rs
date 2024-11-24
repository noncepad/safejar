use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;
use anchor_lang::solana_program::hash::HASH_BYTES;

use crate::errors::TreasuryError;
use crate::rule::{generic_hash, Rule, RULE_RATE_LIMITER};
use crate::spend::{SpendState, TransferContext};
use crate::{nplog, RuleAddRateLimiter, SpendProcessRateLimiter};

impl<'info> RuleAddRateLimiter<'info> {
    pub fn process(&mut self, max_spend: u64, delta_slot: u64) -> ProgramResult {
        let rule = RateLimiter::new(&self.mint.key(), max_spend, delta_slot)?;
        if self.accumulator.add(&rule).is_err() {
            return Err(ProgramError::Custom(TreasuryError::RuleAddFail.into()));
        }
        Ok(())
    }
}

impl<'info> SpendProcessRateLimiter<'info> {
    pub fn process(&mut self, max_spend: u64, delta_slot: u64) -> ProgramResult {
        //msg!("sprl - 1");
        let rule = RateLimiter::new(&self.required_mint.key(), max_spend, delta_slot)?;
        //msg!("sprl - 2");
        self.request.process(&rule)?;
        //msg!("sprl - 3");
        Ok(())
    }
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct RateLimiter {
    pub mint: Pubkey,
    // rate numerator
    pub max_spend: u64,
    // rate denominator
    pub delta_slot: u64,
}

impl RateLimiter {
    pub fn new(mint: &Pubkey, max_spend: u64, delta_slot: u64) -> Result<Self> {
        if max_spend == 0 {
            return Err(TreasuryError::RateLimiterMaxSpendCannotBeZero.into());
        }
        if delta_slot < 100 {
            return Err(TreasuryError::RateLimiterDeltaSlotMustBeGreaterThan100.into());
        }
        Ok(Self {
            mint: mint.clone(),
            max_spend,
            delta_slot,
        })
    }
}

impl<'b> Rule<'b> for RateLimiter {
    fn id(&self) -> u8 {
        return RULE_RATE_LIMITER;
    }

    fn process<'a>(&'a self, state: &mut SpendState, tx_ctx: &TransferContext) -> Result<()> {
        nplog!("rl - 1");
        let space = state.find(&tx_ctx.mint)?;
        nplog!("rl - 2");
        if tx_ctx.slot < space.last_slot {
            nplog!("rl - 3");
            return Err(TreasuryError::RuleRateLimiterSlotOutOfOrder.into());
        }

        nplog!(
            "last slot {} vs context slot {}; delta {}; last spend: {}; next spend: {}",
            space.last_slot,
            tx_ctx.slot,
            tx_ctx.slot - space.last_slot + 1,
            space.last_spend,
            tx_ctx.amount,
        );
        let spent;
        // check if the last spend is irrelevant (too old)
        if tx_ctx.slot < space.last_slot + self.delta_slot {
            spent = space.last_spend + tx_ctx.amount;
        } else {
            spent = tx_ctx.amount;
        }
        if self.max_spend <= spent {
            return Err(TreasuryError::RuleRateLimiterCannotExceedSpendLimit.into());
        }

        nplog!("rl - 7 - rate limit",);

        Ok(())
    }

    fn hash<'a>(&'a self, index: u8, prev_hash: &'a [u8]) -> Result<[u8; HASH_BYTES]> {
        let mut x = [0u8; std::mem::size_of::<RateLimiter>()];
        let mut cursor = std::io::Cursor::new(x.as_mut());
        self.serialize(&mut cursor)?;
        //msg!("_______+++++rule({})={:X?}",x.len(),&x);
        return Ok(generic_hash(&index, &x, prev_hash));
    }
}
