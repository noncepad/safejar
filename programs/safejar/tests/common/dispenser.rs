use std::{cell::RefCell, rc::Rc};

use anchor_lang::{
    accounts::program, system_program, AccountDeserialize, AnchorDeserialize, AnchorSerialize,
    InstructionData, ToAccountMetas,
};
use anchor_spl::{associated_token, token::ID as TokenProgramID};
use hex::encode;
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
        CompleteSpendRequestDirect as DataCompleteSpendRequestDirect,
        CreateRuleAccumulator as DataCreateRuleAccumulator,
        CreateSpendRequestDirect as DataCreateSpendRequestDirect, Delegate as DataDelegate,
        RuleAddAuthorizationConstraint as DataRuleAddAuthorizationConstraint,
        RuleAddProgramConstraint as DataRuleAddProgramConstraint,
        RuleAddRateLimiter as DataRuleAddRateLimiter,
        RuleProcessAuthorizationConstraint as DataRuleProcessAuthorizationConstraint,
        RuleProcessRateLimiter as DataRuleProcessRateLimiter,
        RuleProcessSweep as DataRuleProcessSweep,
    },
    nplog,
    rule::{Rule, RuleAccumulator},
    ruleauthconstr::{AuthorizationConstraint, AuthorizationConstraintOnly},
    ruleprogconstr::ProgramConstraint,
    ruleratelimiter::RateLimiter,
    spend::{SpendRequest, SpendState, TransferContext},
    tree::{deserialize, serialize, Node},
    ApproveDelegation,
};

use super::{
    basic::{airdrop, send_tx, update_blockhash, SignerList},
    controller::ControllerCreator,
    errors::{CommonError, CustomError},
};

pub struct Dispenser<'a> {
    has_rule_set: bool,
    owner: Pubkey,
    controller: Pubkey,
    rule_list: Vec<Box<dyn DispenserRule<'a>>>,
    max_token_track: u8,
    tree: Rc<RefCell<Node>>,
    rule_count: u8,
}

// this is a Rule, but also we add a function to get instructions
pub trait DispenserRule<'b> {
    fn rule<'a>(&self) -> Box<dyn Rule<'a>>;
    fn add_ix<'a>(&self, accumulator: &Pubkey, owner: &Pubkey) -> Instruction;
    fn spend_ix<'a>(
        &self,
        request: &Pubkey,
        linker: &Pubkey,
        keypair_list: &Vec<Keypair>,
    ) -> Instruction;
}

impl<'a> Dispenser<'a> {
    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn new(
        owner: &Pubkey,
        max_token_track: u8,
        tree_data: &Vec<u8>,
    ) -> Result<Self, CustomError> {
        let (tree, rule_count) = match deserialize(tree_data) {
            Ok((t2, c2)) => match t2 {
                Some(t3) => (t3, c2),
                None => {
                    return Err(CustomError::code::<std::io::Error>(
                        CommonError::Unknown,
                        "".to_owned(),
                    ))
                }
            },
            Err(err) => {
                println!("tree error: {}", err);
                return Err(CustomError::code::<std::io::Error>(
                    CommonError::Unknown,
                    "".to_owned(),
                ));
            }
        };

        return Ok(Self {
            has_rule_set: false,
            controller: controller_id(owner),
            owner: owner.clone(),
            rule_list: Vec::new(),
            tree,
            rule_count,
            max_token_track,
        });
    }

    pub fn rule_add2(&mut self, rule: Box<dyn DispenserRule<'a>>) -> Result<(), CustomError> {
        if self.has_rule_set {
            return Err(CustomError::code::<std::io::Error>(
                CommonError::Unknown,
                "".to_owned(),
            ));
        }
        self.rule_list.push(rule);
        Ok(())
    }

    // stop adding rules
    pub fn rule_stop(&mut self) -> Result<(), CustomError> {
        if self.rule_list.len() as u8 != self.rule_count {
            return Err(CustomError::code::<std::io::Error>(
                CommonError::Unknown,
                "".to_owned(),
            ));
        }
        self.has_rule_set = true;
        Ok(())
    }

    pub fn delegate(
        &self,
        linker: &Pubkey,
        ix_list: &mut Vec<Instruction>,
    ) -> Result<Keypair, CustomError> {
        if !self.has_rule_set {
            println!("no rule set");
            return Err(CustomError::code::<std::io::Error>(
                CommonError::Unknown,
                "does not match".to_owned(),
            ));
        }
        // this is an ephemeral key only used inside a single transaction
        let accumulator_kp = Keypair::new();
        let accumulator = accumulator_kp.pubkey();
        if self.rule_list.len() as u8 != self.rule_count {
            println!(
                "rule list count does not match: {} vs {}",
                self.rule_list.len(),
                self.rule_count
            );
            return Err(CustomError::code::<std::io::Error>(
                CommonError::Unknown,
                "does not match".to_owned(),
            ));
        }
        ix_list.push(self.inside_accumulator(linker, &accumulator));
        for x in &self.rule_list {
            ix_list.push(x.add_ix(&accumulator, &self.owner))
        }
        ix_list.push(self.inside_delegate(&accumulator, linker)?);
        ix_list.push(self.approve_delegation()?);

        Ok(accumulator_kp)
    }

    pub fn delegation_id(&self) -> Result<Pubkey, CustomError> {
        let h = self.hash()?;

        return Ok(delegation_id(&self.controller, &h));
    }

    pub fn hash(&self) -> Result<[u8; 32], CustomError> {
        let root = self.tree.clone();
        let treedata = root.borrow().serialize();
        println!("hash - 1a - tree data; {}", encode(&treedata));
        let (_r2, c2) = safejar::tree::deserialize(&treedata).unwrap();
        if c2 != self.rule_list.len() as u8 {
            panic!("wrong count {} vs {}", c2, self.rule_list.len())
        }

        let mut ra = match RuleAccumulator::new(&self.controller, &treedata) {
            Ok(x) => x,
            Err(err) => {
                println!("rule accumulator init {}", err);
                return Err(CustomError::new(CommonError::Unknown, err));
            }
        };
        for r in &self.rule_list {
            match ra.add(r.rule().as_ref()) {
                Ok(_) => {}
                Err(err) => {
                    return Err(CustomError::new(CommonError::Unknown, err));
                }
            };
        }

        println!("hash - 5");
        return Ok(ra.hash);
    }

    /// Returns a request signer.
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn spend(
        &self,
        keypair_list: &mut Vec<Keypair>,
        ix_list: &mut Vec<Instruction>,
        fee_payer: &Keypair,
        destination_owner: &Pubkey,
        mint: &Pubkey,
        amount: u64,
    ) -> Result<Keypair, CustomError> {
        let request_signer = Keypair::new();

        let delegation = self.delegation_id()?;
        let delegation_vault =
            anchor_spl::associated_token::get_associated_token_address(&delegation, mint);
        let destination_vault =
            anchor_spl::associated_token::get_associated_token_address(destination_owner, mint);

        println!(
            "doing spend from delegation {} {} to {} {}",
            delegation, delegation_vault, destination_owner, destination_vault
        );
        ix_list.push(self.ix_spend_request(
            &request_signer.pubkey(),
            &fee_payer.pubkey(),
            mint,
            destination_owner,
            &destination_vault,
            &delegation,
            &delegation_vault,
            amount,
        )?);

        for r in &self.rule_list {
            ix_list.push(r.spend_ix(&request_signer.pubkey(), &fee_payer.pubkey(), keypair_list))
        }
        ix_list.push(self.ix_spend_complete(
            &fee_payer.pubkey(),
            &request_signer.pubkey(),
            &destination_vault,
            &delegation,
            &delegation_vault,
        )?);

        Ok(request_signer)
    }

    fn ix_spend_request(
        &self,
        request: &Pubkey,
        linker: &Pubkey,
        mint: &Pubkey,
        destination_owner: &Pubkey,
        destination_vault: &Pubkey,
        delegation: &Pubkey,
        delegation_vault: &Pubkey,
        amount: u64,
    ) -> Result<Instruction, CustomError> {
        let tree = serialize(Some(self.tree.clone()));
        println!(
            "spend request delegation {} delegation vault {}",
            delegation, delegation_vault
        );
        return Ok(Instruction::new_with_bytes(
            safejar::ID,
            DataCreateSpendRequestDirect { amount, tree }
                .data()
                .as_ref(),
            vec![
                AccountMeta::new(delegation.clone(), false),
                AccountMeta::new(request.clone(), true),
                AccountMeta::new(delegation_vault.clone(), false),
                AccountMeta::new(destination_vault.clone(), false),
                AccountMeta::new(destination_owner.clone(), false),
                AccountMeta::new(mint.clone(), false),
                AccountMeta::new(linker.clone(), true),
                AccountMeta::new(rent_id, false),
                AccountMeta::new(system_program::ID, false),
                AccountMeta::new(clock_id, false),
                AccountMeta::new(TokenProgramID, false),
                AccountMeta::new(associated_token::ID, false),
            ],
        ));
    }

    fn ix_spend_complete(
        &self,
        fee_payer: &Pubkey,
        request: &Pubkey,
        destination_vault: &Pubkey,
        delegation: &Pubkey,
        delegation_vault: &Pubkey,
    ) -> Result<Instruction, CustomError> {
        return Ok(Instruction::new_with_bytes(
            safejar::ID,
            DataCompleteSpendRequestDirect {}.data().as_ref(),
            vec![
                AccountMeta::new(request.clone(), false),
                AccountMeta::new(delegation.clone(), false),
                AccountMeta::new(delegation_vault.clone(), false),
                AccountMeta::new(destination_vault.clone(), false),
                AccountMeta::new(system_program::ID, false),
                AccountMeta::new(TokenProgramID, false),
                AccountMeta::new(fee_payer.clone(), true),
            ],
        ));
    }

    fn inside_accumulator(&self, linker: &Pubkey, accumulator: &Pubkey) -> Instruction {
        let tree_data = serialize(Some(self.tree.clone()));
        return Instruction::new_with_bytes(
            safejar::ID,
            DataCreateRuleAccumulator { tree: tree_data }
                .data()
                .as_ref(),
            vec![
                AccountMeta::new(self.controller.clone(), false),
                AccountMeta::new(accumulator.clone(), true),
                AccountMeta::new(self.owner.clone(), true),
                AccountMeta::new(linker.clone(), true),
                AccountMeta::new(rent_id, false),
                AccountMeta::new(system_program::ID, false),
                AccountMeta::new(TokenProgramID, false),
            ],
        );
    }

    fn inside_delegate(
        &self,
        accumulator: &Pubkey,
        linker: &Pubkey,
    ) -> Result<Instruction, CustomError> {
        let delegation = self.delegation_id()?;
        nplog!("max token track {}", self.max_token_track);
        return Ok(Instruction::new_with_bytes(
            safejar::ID,
            DataDelegate {
                max_spend_state: self.max_token_track,
            }
            .data()
            .as_ref(),
            vec![
                AccountMeta::new(self.controller.clone(), false),
                AccountMeta::new(delegation.clone(), false),
                AccountMeta::new(accumulator.clone(), false),
                AccountMeta::new(linker.clone(), true),
                AccountMeta::new(self.owner.clone(), true),
                AccountMeta::new(rent_id, false),
                AccountMeta::new(system_program::ID, false),
                AccountMeta::new(clock_id, false),
            ],
        ));
    }

    fn approve_delegation(&self) -> Result<Instruction, CustomError> {
        println!("approve_ix - 1");
        let delegation = self.delegation_id()?;
        println!("approve_ix - 2");
        return Ok(Instruction::new_with_bytes(
            safejar::ID,
            DataApproveDelegation {}.data().as_ref(),
            vec![
                AccountMeta::new(self.controller.clone(), false),
                AccountMeta::new(delegation, false),
                AccountMeta::new(self.owner.clone(), true),
            ],
        ));
    }
}

pub async fn do_spend<'a>(
    context: &mut ProgramTestContext,
    keypair_list: &mut Vec<Keypair>,
    fee_payer: &Keypair,
    dispenser: &Dispenser<'a>,
    destination_owner: &Pubkey,
    mint: &Pubkey,
    amount: u64,
) -> Result<(), CustomError> {
    update_blockhash(context).await.unwrap();
    //let mut keypair_list = Vec::new();
    let mut ix_list = Vec::new();

    let request = dispenser.spend(
        keypair_list,
        &mut ix_list,
        fee_payer,
        destination_owner,
        mint,
        amount,
    )?;

    keypair_list.push(fee_payer.insecure_clone());
    keypair_list.push(request);
    let sl = SignerList::new(&keypair_list);
    let tx = Transaction::new_signed_with_payer(
        &ix_list,
        Some(&fee_payer.pubkey()),
        &sl,
        context.last_blockhash,
    );

    match context.banks_client.process_transaction(tx).await {
        Ok(_) => Ok(()),
        Err(err) => {
            println!("failed tx {}", err);
            return Err(CustomError::new(CommonError::Unknown, err));
        }
    }
}

// create the delegation account
pub async fn do_delegation<'a>(
    context: &mut ProgramTestContext,
    fee_payer: &Keypair,
    ctr: &ControllerCreator,
    dispenser: &Dispenser<'a>,
) {
    update_blockhash(context).await.unwrap();
    let mut ix_list = Vec::new();
    let accumulator_signer = dispenser
        .delegate(&fee_payer.pubkey(), &mut ix_list)
        .unwrap();
    let tx = Transaction::new_signed_with_payer(
        &ix_list,
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &ctr.owner, &accumulator_signer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();
}
