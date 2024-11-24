//#![cfg(feature = "test-sbf")]

use std::{cell::RefCell, rc::Rc};

use anchor_lang::{
    accounts::program, system_program, AccountDeserialize, AnchorDeserialize, InstructionData,
    ToAccountMetas,
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

use super::errors::CustomError;

pub async fn fetch_delegation(
    context: &mut ProgramTestContext,
    delegation: &Pubkey,
) -> Result<Option<safejar::delegate::Delegation>, CustomError> {
    nplog!("searching delegation {}", delegation);
    let a = context
        .banks_client
        .get_account_with_commitment(
            delegation.clone(),
            solana_sdk::commitment_config::CommitmentLevel::Confirmed,
        )
        .await
        .unwrap();
    if a.is_none() {
        return Ok(None);
    }
    let b = a.unwrap();
    let mut x = &b.data[8..];
    let x = safejar::delegate::Delegation::deserialize(&mut x)?;
    return Ok(Some(x));
}

pub async fn token_balance(context: &mut ProgramTestContext, mint: &Pubkey, owner: &Pubkey) -> u64 {
    let address = get_associated_token_address(owner, mint);
    println!("looking up token balance for {} {}", owner, address);
    let a = context
        .banks_client
        .get_account_with_commitment(
            address.clone(),
            solana_sdk::commitment_config::CommitmentLevel::Confirmed,
        )
        .await
        .unwrap()
        .unwrap();

    let y = TokenAccount::try_deserialize(&mut a.data()).unwrap();
    return y.amount;
}
