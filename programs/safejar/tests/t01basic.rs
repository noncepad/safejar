//#![cfg(feature = "test-sbf")]

use anchor_lang::{
    accounts::program, system_program, AccountDeserialize, AnchorDeserialize, InstructionData,
    ToAccountMetas,
};
use anchor_spl::token::ID as TokenProgramID;
use rand::Rng;
use safejar::{
    self,
    controller::{controller_id, Controller},
    instruction::CreateController,
};
use solana_program::{
    hash::{Hash, HASH_BYTES},
    instruction::{AccountMeta, Instruction},
    sysvar::{clock::ID as clock_id, rent::ID as rent_id},
};
use solana_program_test::{tokio, BanksClientError, ProgramTest, ProgramTestContext};
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
pub mod common;
use common::{
    basic::{airdrop, send_tx},
    controller::ControllerCreator,
};

#[tokio::test]
async fn f01_create_controller() {
    let mut validator = ProgramTest::default();

    validator.add_program("safejar", safejar::ID, None);
    let mut context = validator.start_with_context().await;
    let faucet = Keypair::from_base58_string(context.payer.to_base58_string().as_str());

    let creator = ControllerCreator::new();
    airdrop(&mut context, &creator.owner.pubkey(), 10_000_000)
        .await
        .unwrap();
    assert!(
        100_000
            <= context
                .banks_client
                .get_balance(creator.owner.pubkey())
                .await
                .unwrap()
    );
    send_tx(
        &mut context,
        &[creator.create_ix()],
        &faucet.pubkey(),
        &[&faucet, &creator.owner],
    )
    .await
    .unwrap();
    assert_ne!(
        context.banks_client.get_balance(creator.id).await.unwrap(),
        0
    );

    // download the controller state
    let controller_account = context
        .banks_client
        .get_account_with_commitment(
            creator.id.clone(),
            solana_sdk::commitment_config::CommitmentLevel::Confirmed,
        )
        .await
        .unwrap()
        .unwrap();
    let mut x = &controller_account.data[8..];
    let x = Controller::deserialize(&mut x).unwrap();
    assert_eq!(x.owner, creator.owner.pubkey());
    assert_eq!(x.delegation_count, 0);
}
