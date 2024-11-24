//#![cfg(feature = "test-sbf")]

use anchor_lang::{
    accounts::program, system_program, AccountDeserialize, AnchorDeserialize, InstructionData,
    ToAccountMetas,
};
use anchor_spl::{associated_token, token::ID as TokenProgramID};
use rand::Rng;
use solana_program::{
    hash::{Hash, HASH_BYTES},
    instruction::{AccountMeta, Instruction},
    sysvar::{clock::ID as clock_id, rent::ID as rent_id},
};
use solana_program_test::{tokio, BanksClientError, ProgramTest, ProgramTestContext};
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
use safejar::{
    self,
    controller::{controller_id, Controller},
    instruction::{
        CreateController, CreateRuleAccumulator, TransferToController as DataTransferToController,
        TransferToDelegation as DataTransferToDelegation,
    },
};

use super::{
    basic::{airdrop, send_tx},
    errors::CommonError,
};

pub struct ControllerCreator {
    pub id: Pubkey,
    pub owner: Keypair,
}

impl ControllerCreator {
    pub async fn new_from_context(
        context: &mut ProgramTestContext,
        faucet: &Keypair,
    ) -> Result<Self, BanksClientError> {
        let creator = Self::new();
        airdrop(context, &creator.owner.pubkey(), 10_000_000)
            .await
            .unwrap();
        send_tx(
            context,
            &[creator.create_ix()],
            &faucet.pubkey(),
            &[&faucet, &creator.owner],
        )
        .await?;
        return Ok(creator);
    }

    pub fn new() -> Self {
        let owner = Keypair::new();
        let id2 = controller_id(&owner.pubkey());
        return Self { owner, id: id2 };
    }

    pub fn create_ix(&self) -> Instruction {
        return Instruction::new_with_bytes(
            safejar::ID,
            CreateController {}.data().as_ref(),
            vec![
                AccountMeta::new(self.id.clone(), false),
                AccountMeta::new(self.owner.pubkey().clone(), true),
                AccountMeta::new(self.owner.pubkey().clone(), true),
                AccountMeta::new(rent_id, false),
                AccountMeta::new(system_program::ID, false),
                AccountMeta::new(clock_id, false),
                AccountMeta::new(TokenProgramID, false),
            ],
        );
    }

    pub fn accumulator_ix(&self, accumulator: &Keypair, tree: &Vec<u8>) -> Instruction {
        return Instruction::new_with_bytes(
            safejar::ID,
            CreateRuleAccumulator { tree: tree.clone() }.data().as_ref(),
            vec![
                AccountMeta::new(self.id.clone(), false),
                AccountMeta::new(accumulator.pubkey().clone(), false),
                AccountMeta::new(self.owner.pubkey().clone(), true),
                AccountMeta::new(rent_id, false),
                AccountMeta::new(system_program::ID, false),
                AccountMeta::new(TokenProgramID, false),
            ],
        );
    }

    fn ix_transfer(
        &self,
        to_delegation: bool,
        fee_payer: &Pubkey,
        mint: &Pubkey,
        delegation_id: &Pubkey,
        amount: u64,
    ) -> Instruction {
        let controller_vault =
            spl_associated_token_account::get_associated_token_address(&self.id, mint);
        let delegation_vault =
            spl_associated_token_account::get_associated_token_address(delegation_id, mint);
        if to_delegation {
            println!(
                "controller {} {} to delegation {} {} amount {}",
                self.id, controller_vault, delegation_id, delegation_vault, amount
            );
            return Instruction::new_with_bytes(
                safejar::ID,
                DataTransferToDelegation { amount }.data().as_ref(),
                vec![
                    AccountMeta::new(self.id.clone(), false),
                    AccountMeta::new(controller_vault.clone(), false),
                    AccountMeta::new(delegation_id.clone(), false),
                    AccountMeta::new(delegation_vault, false),
                    AccountMeta::new(mint.clone(), false),
                    AccountMeta::new(self.owner.pubkey(), true),
                    AccountMeta::new(fee_payer.clone(), true),
                    AccountMeta::new(TokenProgramID, false),
                    AccountMeta::new(spl_associated_token_account::ID, false),
                    AccountMeta::new(system_program::ID, false),
                ],
            );
        } else {
            return Instruction::new_with_bytes(
                safejar::ID,
                DataTransferToController { amount }.data().as_ref(),
                vec![
                    AccountMeta::new(self.id.clone(), false),
                    AccountMeta::new(controller_vault.clone(), false),
                    AccountMeta::new(delegation_id.clone(), false),
                    AccountMeta::new(delegation_vault, false),
                    AccountMeta::new(mint.clone(), false),
                    AccountMeta::new(self.owner.pubkey(), true),
                    AccountMeta::new(fee_payer.clone(), true),
                    AccountMeta::new(TokenProgramID, true),
                    AccountMeta::new(spl_associated_token_account::ID, true),
                    AccountMeta::new(system_program::ID, true),
                ],
            );
        }
    }

    pub async fn transfer(
        &self,
        context: &mut ProgramTestContext,
        to_delegation: bool,
        fee_payer: &Keypair,
        mint: &Pubkey,
        delegation_id: &Pubkey,
        amount: u64,
    ) -> Result<(), BanksClientError> {
        send_tx(
            context,
            &[self.ix_transfer(
                to_delegation,
                &fee_payer.pubkey(),
                mint,
                delegation_id,
                amount,
            )],
            &fee_payer.pubkey(),
            &[&fee_payer, &self.owner],
        )
        .await?;

        Ok(())
    }
}
