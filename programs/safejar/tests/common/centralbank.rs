use anchor_lang::{prelude::borsh::BorshSerialize, AccountDeserialize, AnchorSerialize};

use rand::Rng;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_option::COption,
    program_pack::Pack,
    system_program::ID as system_program_id,
    sysvar::rent::ID as rent_id,
};
//use solana_program::{instruction::Instruction, system_instruction::create_account};
use solana_program_test::{ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::{Account, AccountSharedData},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    system_instruction::{allocate, assign, create_account, transfer},
    transaction::Transaction,
};

use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account_idempotent,
    ID as associated_token_account_program_id,
};
use spl_token::{
    self,
    instruction::{
        initialize_mint, mint_to,
        TokenInstruction::{InitializeMint, MintTo},
    },
    state::Mint,
    ID as token_program_id,
};

use super::{basic::update_blockhash, errors::CommonError};

pub struct CentralBank {
    pub id: Pubkey,
    pub auth: Keypair,
    freeze: Keypair,
}

impl CentralBank {
    pub fn new_from_validator(validator: &mut ProgramTest) -> Result<Self, CommonError> {
        let id = Keypair::new();
        let auth = Keypair::new();
        let freeze = Keypair::new();

        let mut data: [u8; Mint::LEN] = [0u8; Mint::LEN];
        match Mint::pack(
            Mint {
                mint_authority: COption::Some(auth.pubkey()),
                supply: 0,
                decimals: 2,
                is_initialized: true,
                freeze_authority: COption::Some(freeze.pubkey()),
            },
            &mut data,
        ) {
            Ok(_) => {}
            Err(_) => return Err(CommonError::Unknown),
        }
        let cb = Self {
            id: id.pubkey(),
            auth,
            freeze,
        };
        validator.add_account(
            id.pubkey(),
            Account {
                lamports: 1_200_000,
                data: Vec::from(data),
                owner: token_program_id.clone(),
                executable: false,
                rent_epoch: 0,
            },
        );

        return Ok(cb);
    }

    pub async fn issue(
        &self,
        context: &mut ProgramTestContext,
        faucet: &Keypair,
        recipient: &Pubkey,
        amount: u64,
    ) -> Result<(), CommonError> {
        let mut list = Vec::new();
        println!("issue {} to recipient {}", amount, recipient);
        self.issue_ix(&mut list, &faucet.pubkey(), &recipient, amount)
            .unwrap();
        update_blockhash(context).await?;

        let tx = Transaction::new_signed_with_payer(
            &list,
            Some(&faucet.pubkey()),
            &[&faucet, &self.auth],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await?;
        Ok(())
    }

    fn issue_ix(
        &self,
        list: &mut Vec<Instruction>,
        fee_payer: &Pubkey,
        recipient: &Pubkey,
        amount: u64,
    ) -> Result<(), CommonError> {
        let token_account = get_associated_token_address(recipient, &self.id);

        list.push(create_associated_token_account_idempotent(
            fee_payer,
            recipient,
            &self.id,
            &token_program_id,
        ));
        match mint_to(
            &token_program_id,
            &self.id,
            &token_account,
            &self.auth.pubkey(),
            &[],
            amount,
        ) {
            Ok(i1) => {
                list.push(i1);
            }
            Err(_) => return Err(CommonError::Unknown),
        }

        Ok(())
    }
}
