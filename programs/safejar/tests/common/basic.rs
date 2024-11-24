use anchor_lang::AccountDeserialize;
use rand::Rng;
use solana_program::instruction::Instruction;
use solana_program_test::{processor, tokio, BanksClientError, ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::AccountSharedData, pubkey::Pubkey, signature::Keypair, signer::Signer,
    signers::Signers, system_instruction, transaction::Transaction,
};

pub struct SignerList<'a> {
    pubkey_list: Vec<Pubkey>,
    keypair_list: Vec<&'a Keypair>,
}

impl<'a> SignerList<'a> {
    pub fn new(keypair_list: &'a Vec<Keypair>) -> Self {
        let mut ans = Self {
            pubkey_list: Vec::new(),
            keypair_list: Vec::new(),
        };

        for kp in keypair_list {
            ans.keypair_list.push(&kp);
            ans.pubkey_list.push(kp.pubkey());
        }
        return ans;
    }
}

impl<'a> Signers for SignerList<'a> {
    fn pubkeys(&self) -> Vec<Pubkey> {
        let mut x = Vec::new();
        for pk in &self.pubkey_list {
            x.push(pk.clone());
        }
        return x;
    }

    fn try_pubkeys(&self) -> Result<Vec<Pubkey>, solana_sdk::signer::SignerError> {
        return Ok(self.pubkeys());
    }

    fn sign_message(&self, message: &[u8]) -> Vec<solana_sdk::signature::Signature> {
        let mut sigs = Vec::new();
        for kp in &self.keypair_list {
            sigs.push(kp.sign_message(message))
        }
        return sigs;
    }

    fn try_sign_message(
        &self,
        message: &[u8],
    ) -> Result<Vec<solana_sdk::signature::Signature>, solana_sdk::signer::SignerError> {
        return Ok(self.sign_message(message));
    }

    fn is_interactive(&self) -> bool {
        return false;
    }
}

pub(crate) fn add_account(validator: &mut ProgramTest, balance: u64) -> Keypair {
    let keypair = Keypair::new();
    let account = AccountSharedData::new(balance, 0, &solana_sdk::system_program::id());
    validator.add_account(keypair.pubkey(), account.into());

    return keypair;
}

pub(crate) async fn send_tx(
    context: &mut ProgramTestContext,
    instructions: &[Instruction],
    payer: &Pubkey,
    signers: &[&Keypair],
) -> Result<(), BanksClientError> {
    update_blockhash(context).await?;
    let tx = Transaction::new_signed_with_payer(
        instructions,
        Some(payer),
        signers,
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction_with_commitment(
            tx,
            solana_sdk::commitment_config::CommitmentLevel::Finalized,
        )
        .await?;
    Ok(())
}

pub(crate) async fn airdrop(
    context: &mut ProgramTestContext,
    receiver: &Pubkey,
    amount: u64,
) -> Result<(), BanksClientError> {
    let rent = context.banks_client.get_rent().await.unwrap();
    let mb = rent.minimum_balance(0);
    update_blockhash(context).await?;
    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::transfer(
            &context.payer.pubkey(),
            receiver,
            amount + mb,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();
    Ok(())
}

pub(crate) async fn update_blockhash(
    context: &mut ProgramTestContext,
) -> Result<(), BanksClientError> {
    let current_slot = context.banks_client.get_root_slot().await?;
    context
        .warp_to_slot(current_slot + 5)
        .map_err(|_| BanksClientError::ClientError("Warp to slot failed!"))?;
    Ok(())
}
