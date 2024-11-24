use std::{cell::RefCell, rc::Rc};

use anchor_lang::{
    accounts::program, system_program, AccountDeserialize, AnchorDeserialize, AnchorSerialize,
    InstructionData, ToAccountMetas,
};
use anchor_spl::token::ID as TokenProgramID;
use rand::Rng;
use solana_program::{
    hash::{Hash, HASH_BYTES},
    instruction::{AccountMeta, Instruction},
    sysvar::{clock::ID as clock_id, rent::ID as rent_id},
};
use solana_program_test::{
    tokio::{self, sync::watch::Ref},
    BanksClientError, ProgramTest, ProgramTestContext,
};
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};
use safejar::{
    self,
    controller::{controller_id, Controller},
    delegate::delegation_id,
    instruction::{
        ApproveDelegation as DataApproveDelegation,
        CreateRuleAccumulator as DataCreateRuleAccumulator, Delegate as DataDelegate,
        RuleAddAuthorizationConstraint as DataRuleAddAuthorizationConstraint,
        RuleAddProgramConstraint as DataRuleAddProgramConstraint,
        RuleAddRateLimiter as DataRuleAddRateLimiter,
        RuleProcessAuthorizationConstraint as DataRuleProcessAuthorizationConstraint,
        RuleProcessNonAuthorizationConstraint as DataRuleProcessNonAuthorizationConstraint,
        RuleProcessRateLimiter as DataRuleProcessRateLimiter,
    },
    rule::{Rule, RuleAccumulator},
    ruleauthconstr::{
        AuthorizationConstraint as RAuthorizationConstraint,
        AuthorizationConstraintOnly as RAuthorizationConstraintOnly,
    },
    ruleprogconstr::ProgramConstraint,
    ruleratelimiter::RateLimiter as RRateLimiter,
    tree::Node,
    ApproveDelegation,
};

use super::{
    basic::{airdrop, send_tx, update_blockhash},
    controller::ControllerCreator,
    dispenser::DispenserRule,
    errors::CommonError,
};

#[derive(Clone)]
pub struct RateLimiter {
    pub x: RRateLimiter,
}

impl<'b> DispenserRule<'b> for RateLimiter {
    fn rule<'a>(&self) -> Box<dyn Rule<'a>> {
        // the will_sign is irrelevant here
        return Box::new(self.x.clone());
    }

    fn add_ix<'a>(&self, accumulator: &Pubkey, owner: &Pubkey) -> Instruction {
        let rl = DataRuleAddRateLimiter {
            max_spend: self.x.max_spend,
            delta_slot: self.x.delta_slot,
        };
        let controller = controller_id(owner);
        return Instruction::new_with_bytes(
            safejar::ID,
            rl.data().as_ref(),
            vec![
                AccountMeta::new(controller, false),
                AccountMeta::new(accumulator.clone(), false),
                AccountMeta::new(owner.clone(), true),
                AccountMeta::new(self.x.mint.clone(), false),
            ],
        );
    }

    fn spend_ix<'a>(
        &self,
        request: &Pubkey,
        linker: &Pubkey,
        _keypair_list: &Vec<Keypair>,
    ) -> Instruction {
        let rl = DataRuleProcessRateLimiter {
            max_spend: self.x.max_spend,
            delta_slot: self.x.delta_slot,
        };
        return Instruction::new_with_bytes(
            safejar::ID,
            rl.data().as_ref(),
            vec![
                AccountMeta::new(request.clone(), false),
                AccountMeta::new(self.x.mint.clone(), false),
                AccountMeta::new(linker.clone(), true),
            ],
        );
    }
}
