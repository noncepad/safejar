//#![cfg(feature = "test-sbf")]

use std::{cell::RefCell, rc::Rc};

use anchor_lang::{
    accounts::program, prelude::borsh::de, system_program, AccountDeserialize, AnchorDeserialize,
    InstructionData, ToAccountMetas,
};
use anchor_spl::{
    associated_token::get_associated_token_address,
    token::{TokenAccount, ID as TokenProgramID},
};
use rand::Rng;
use solana_program::{
    hash::{Hash, HASH_BYTES},
    instruction::{AccountMeta, Instruction},
    sysvar::{clock::ID as clock_id, rent::ID as rent_id},
};
use solana_program_test::{tokio, BanksClientError, ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::{ReadableAccount, WritableAccount},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    signers::Signers,
    transaction::Transaction,
};
use safejar::{
    self,
    controller::{controller_id, Controller},
    delegate::Delegation as BDelegation,
    instruction::CreateController,
    nplog,
    rule::Rule,
    ruleauthconstr::AuthorizationConstraintOnly,
    ruleratelimiter::RateLimiter,
    tree::{serialize, Node},
};

pub mod common;
use common::{
    basic::{airdrop, send_tx},
    centralbank::CentralBank,
    controller::ControllerCreator,
    errors::CustomError,
};

use crate::common::{
    basic::update_blockhash,
    dispenser::{do_delegation, do_spend, Dispenser},
    errors::CommonError,
    rpc::{fetch_delegation, token_balance},
    ruleac, rulerl, ruleswp,
};

/// Test the authorization constraint and rate limit.
///
/// # Panics
///
/// Panics if .
#[tokio::test]
async fn f02_1_delegation_simple() {
    let mut validator = ProgramTest::default();
    validator.add_program("safejar", safejar::ID, None);
    // create a new token so we can issue our self tokens during the test.
    let cb: CentralBank = CentralBank::new_from_validator(&mut validator).unwrap();
    // we get SOL from here via "airdrop"
    let mut context: ProgramTestContext = validator.start_with_context().await;
    // give ourselves 10 SOL
    let fee_payer = Keypair::new();
    let ctr: ControllerCreator = prepare_controller(&mut context, &fee_payer, &cb).await;

    // create the dispenser
    let tree_data = serialize(Some(f02_1_make_tree()));
    let mut dispenser = Dispenser::new(&ctr.owner.pubkey(), 1, &tree_data).unwrap();
    // rule 1
    let mut left_over_before_slot = 750_000;
    let max_spend = left_over_before_slot;
    let delta_slot = 500;
    let rl = Box::new(rulerl::RateLimiter {
        x: RateLimiter {
            mint: cb.id.clone(),
            max_spend,
            delta_slot,
        },
    });
    dispenser.rule_add2(rl.clone()).unwrap();
    // rule 2
    let authorizer1 = Keypair::new();
    let ac1 = Box::new(ruleac::AuthorizationConstraint::new(
        AuthorizationConstraintOnly {
            required_authorizer: authorizer1.pubkey(),
        },
    ));
    dispenser.rule_add2(ac1.clone()).unwrap();

    // there are no more rules to add
    dispenser.rule_stop().unwrap();

    println!("fee payer {}", fee_payer.pubkey());
    println!("controller owner {}", ctr.owner.pubkey());
    do_delegation(&mut context, &fee_payer, &ctr, &dispenser).await;

    let delegation_account = context
        .banks_client
        .get_account_with_commitment(
            dispenser.delegation_id().unwrap(),
            solana_sdk::commitment_config::CommitmentLevel::Confirmed,
        )
        .await
        .unwrap()
        .unwrap();
    let mut x = &delegation_account.data[8..];
    let x = BDelegation::deserialize(&mut x).unwrap();

    let hash = dispenser.hash().unwrap();
    for i in 0..32 {
        if hash[i] != x.rule_set_hash[i] {
            panic!("hash value is different");
        }
    }

    // give money to the controller
    let tx_amt_1: u64 = 10_324_000;
    cb.issue(&mut context, &fee_payer, &ctr.id, 2 * tx_amt_1)
        .await
        .unwrap();
    if token_balance(&mut context, &cb.id, &ctr.id).await == 0 {
        panic!("failed to issue to controller {}", ctr.id);
    }
    let delegation_id = dispenser.delegation_id().unwrap();

    ctr.transfer(
        &mut context,
        true,
        &fee_payer,
        &cb.id,
        &dispenser.delegation_id().unwrap(),
        tx_amt_1,
    )
    .await
    .unwrap();

    if token_balance(&mut context, &cb.id, &delegation_id).await == 0 {
        panic!("failed to transfer balance to {}", &delegation_id)
    }
    let mynextdestowner = Keypair::new();
    let mut keypair_list = Vec::new();
    match do_spend(
        &mut context,
        &mut keypair_list,
        &fee_payer,
        &dispenser,
        &mynextdestowner.pubkey(),
        &cb.id,
        tx_amt_1 / 8,
    )
    .await
    {
        Ok(_) => {
            panic!("spend succeeded withouth authorizer signature");
        }
        Err(err) => {
            if !err.to_string().contains("0x1774") {
                panic!("failed to return correct error");
            } else {
                nplog!("returned correct error message");
            }
        }
    };

    let tx_amt_2 = tx_amt_1 / 16;

    let start_slot = context.banks_client.get_root_slot().await.unwrap();
    let mut slot = start_slot;
    let mut have_failed_from_rate_limit = false;
    // loop until we hit the rate limit
    let mut success_count = 0;
    while slot < start_slot + delta_slot {
        nplog!(
            "remaining tokens {} slot: start {} current {}; delta {}; max {}",
            left_over_before_slot,
            start_slot,
            slot,
            delta_slot,
            max_spend,
        );
        keypair_list = Vec::new();
        keypair_list.push(authorizer1.insecure_clone());
        match do_spend(
            &mut context,
            &mut keypair_list,
            &fee_payer,
            &dispenser,
            &mynextdestowner.pubkey(),
            &cb.id,
            tx_amt_2,
        )
        .await
        {
            Ok(_) => {
                nplog!("spend successful");
                success_count += 1;
            }
            Err(err) => {
                if err.to_string().contains("0x1774") {
                    have_failed_from_rate_limit = true;
                    slot = context.banks_client.get_root_slot().await.unwrap();
                    break;
                } else {
                    panic!("failed with: {}", err)
                }
            }
        };

        let a_delegation = fetch_delegation(&mut context, &dispenser.delegation_id().unwrap())
            .await
            .unwrap()
            .unwrap();
        let mut check_spend = None;
        for x in &a_delegation.state.list {
            nplog!("checking spend state slot {}", x.mint);
            if x.is_blank() {
                continue;
            }
            if x.mint != cb.id.clone() {
                continue;
            }
            check_spend = Some(x.last_spend);
            break;
        }
        if check_spend.unwrap() != tx_amt_2 {
            panic!(
                "spending check is wrong: {} vs {}",
                check_spend.unwrap(),
                tx_amt_2
            );
        }
        left_over_before_slot = token_balance(&mut context, &cb.id, &delegation_id).await;
        slot = context.banks_client.get_root_slot().await.unwrap();
        nplog!("slot start {} current {}", start_slot, slot);
    }
    if !have_failed_from_rate_limit {
        panic!("we failed to reach rate limit");
    }
    if success_count == 0 {
        panic!("failed to spend any money");
    }

    have_failed_from_rate_limit = false;

    // warp until we know for sure the rate limit should success
    nplog!("warping from slot {} to {}", slot, start_slot + delta_slot);
    context.warp_to_slot(start_slot + delta_slot).unwrap();
    slot = context.banks_client.get_root_slot().await.unwrap();
    nplog!(
        "final: remaining tokens {} slot: start {} current {}; delta {}",
        left_over_before_slot,
        start_slot,
        slot,
        delta_slot,
    );
    keypair_list = Vec::new();
    keypair_list.push(authorizer1.insecure_clone());
    match do_spend(
        &mut context,
        &mut keypair_list,
        &fee_payer,
        &dispenser,
        &mynextdestowner.pubkey(),
        &cb.id,
        tx_amt_2,
    )
    .await
    {
        Ok(_) => {
            nplog!("spend successful");
        }
        Err(err) => {
            if err.to_string().contains("0x1774") {
                have_failed_from_rate_limit = true;
            } else {
                panic!("failed with: {}", err)
            }
        }
    };
    if have_failed_from_rate_limit {
        panic!("we should not have failed from rate limit");
    }

    let a_delegation = fetch_delegation(&mut context, &dispenser.delegation_id().unwrap())
        .await
        .unwrap()
        .unwrap();
    let mut check_spend = None;
    for x in &a_delegation.state.list {
        nplog!("checking spend state slot {}", x.mint);
        if x.is_blank() {
            continue;
        }
        if x.mint != cb.id.clone() {
            continue;
        }
        check_spend = Some(x.last_spend);
        break;
    }
    if check_spend.unwrap() != tx_amt_2 {
        panic!(
            "spending check is wrong: {} vs {}",
            check_spend.unwrap(),
            tx_amt_2
        );
    }
}

fn f02_1_make_tree() -> Rc<RefCell<Node>> {
    let mut i: u8 = 0;
    let left = Rc::new(RefCell::new(Node::new()));
    left.borrow_mut().set_i(i);
    i += 1;
    let right = Rc::new(RefCell::new(Node::new()));
    right.borrow_mut().set_i(i);
    return Rc::new(RefCell::new(Node::new_with_children(true, &left, &right)));
}

#[tokio::test]
async fn f02_2_delegation_simplev1() {
    let mut validator = ProgramTest::default();
    validator.add_program("safejar", safejar::ID, None);
    // create a new token so we can issue our self tokens during the test.
    let cb: CentralBank = CentralBank::new_from_validator(&mut validator).unwrap();
    // we get SOL from here via "airdrop"
    let mut context: ProgramTestContext = validator.start_with_context().await;
    // give ourselves 10 SOL
    let fee_payer = Keypair::new();
    let ctr: ControllerCreator = prepare_controller(&mut context, &fee_payer, &cb).await;

    // create the dispenser
    let tree_data = serialize(Some(f02_2_make_tree()));
    let mut dispenser = Dispenser::new(&ctr.owner.pubkey(), 1, &tree_data).unwrap();
    // rule 1

    let rl = Box::new(rulerl::RateLimiter {
        x: RateLimiter {
            mint: cb.id.clone(),
            max_spend: 10_000_000,
            delta_slot: 500,
        },
    });
    dispenser.rule_add2(rl.clone()).unwrap();
    // rule 2
    let authorizer1 = Keypair::new();
    let ac1 = Box::new(ruleac::AuthorizationConstraint::new(
        AuthorizationConstraintOnly {
            required_authorizer: authorizer1.pubkey(),
        },
    ));
    dispenser.rule_add2(ac1.clone()).unwrap();

    let swp1 = Box::new(ruleswp::Sweep::new(&fee_payer.pubkey(), &cb.id, 10_000));
    dispenser.rule_add2(swp1.clone()).unwrap();

    // there are no more rules to add
    dispenser.rule_stop().unwrap();

    println!("fee payer {}", fee_payer.pubkey());
    println!("controller owner {}", ctr.owner.pubkey());
    do_delegation(&mut context, &fee_payer, &ctr, &dispenser).await;

    println!("f - 7");
}

// sweepv1() || (ac &&  rl)
fn f02_2_make_tree() -> Rc<RefCell<Node>> {
    let mut i: u8 = 0;
    let ac = Rc::new(RefCell::new(Node::new()));
    ac.borrow_mut().set_i(i);
    i += 1;
    let rl = Rc::new(RefCell::new(Node::new()));
    rl.borrow_mut().set_i(i);
    i += 1;
    let join1 = Rc::new(RefCell::new(Node::new_with_children(true, &ac, &rl)));

    let swp = Rc::new(RefCell::new(Node::new()));
    swp.borrow_mut().set_i(i);
    //i += 1;
    let join1 = Rc::new(RefCell::new(Node::new_with_children(true, &swp, &join1)));

    return join1;
}

async fn prepare_controller(
    context: &mut ProgramTestContext,
    fee_payer: &Keypair,
    cb: &CentralBank,
) -> ControllerCreator {
    airdrop(context, &fee_payer.pubkey(), 10 * 100_000_000)
        .await
        .unwrap();
    // create tokens, give some to our wallet
    cb.issue(context, fee_payer, &fee_payer.pubkey(), 1000)
        .await
        .unwrap();

    let ctr: ControllerCreator = ControllerCreator::new_from_context(context, &fee_payer)
        .await
        .unwrap();

    return ctr;
}
