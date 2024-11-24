use std::{cell::RefCell, rc::Rc};

use anchor_lang::{
    accounts::program, system_program, AccountDeserialize, AnchorDeserialize, AnchorSerialize,
    InstructionData, ToAccountMetas,
};
use anchor_spl::{self, token::ID as TokenProgramID};
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
        RuleAddRateLimiter as DataRuleAddRateLimiter, RuleAddSweep as DataRuleAddSweep,
        RuleProcessAuthorizationConstraint as DataRuleProcessAuthorizationConstraint,
        RuleProcessNonAuthorizationConstraint as DataRuleProcessNonAuthorizationConstraint,
        RuleProcessRateLimiter as DataRuleProcessRateLimiter,
        RuleProcessSweep as DataRuleProcessSweep,
    },
    rule::{Rule, RuleAccumulator},
    ruleauthconstr::{
        AuthorizationConstraint as RAuthorizationConstraint,
        AuthorizationConstraintOnly as RAuthorizationConstraintOnly,
    },
    ruleprogconstr::ProgramConstraint,
    ruleratelimiter::RateLimiter,
    rulesweep::{Sweep as RSweep, SweepOnly as RSweepOnly},
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
pub struct Sweep {
    pub x: RSweep,
}

impl Sweep {
    pub fn new(destination_owner: &Pubkey, mint: &Pubkey, min_bal: u64) -> Self {
        let destination =
            anchor_spl::associated_token::get_associated_token_address(destination_owner, mint);
        return Self {
            x: RSweep {
                destination,
                mint: mint.clone(),
                balance: 0,
                min_bal,
            },
        };
    }
}

impl<'b> DispenserRule<'b> for Sweep {
    fn rule<'a>(&self) -> Box<dyn Rule<'a>> {
        // the will_sign is irrelevant here

        return Box::new(self.x.clone());
    }

    fn add_ix<'a>(&self, accumulator: &Pubkey, owner: &Pubkey) -> Instruction {
        return Instruction::new_with_bytes(
            safejar::ID,
            DataRuleAddSweep {
                min_balance: self.x.min_bal,
            }
            .data()
            .as_ref(),
            vec![
                AccountMeta::new(controller_id(owner), false),
                AccountMeta::new(accumulator.clone(), false),
                AccountMeta::new(owner.clone(), true),
                AccountMeta::new(self.x.destination.clone(), false),
            ],
        );
    }

    fn spend_ix<'a>(
        &self,
        request: &Pubkey,
        linker: &Pubkey,
        _keypair_list: &Vec<Keypair>,
    ) -> Instruction {
        return Instruction::new_with_bytes(
            safejar::ID,
            DataRuleProcessSweep {
                min_balance: self.x.min_bal,
            }
            .data()
            .as_ref(),
            vec![
                AccountMeta::new(request.clone(), false),
                AccountMeta::new(self.x.destination.clone(), false),
                AccountMeta::new(linker.clone(), true),
            ],
        );
    }
}
