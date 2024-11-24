use std::cmp::Ordering;

use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};

use anchor_spl::{
    token::{      
        Token, Mint, TokenAccount,
        spl_token::native_mint::ID as WSOL,
    }, 
    associated_token::AssociatedToken,
    
};


pub mod controller;
pub mod delegate;
pub mod rule;
pub mod ruleratelimiter;
pub mod ruleprogconstr;
pub mod ruleauthconstr;
pub mod rulesweep;
pub mod rulemaxbal;
pub mod spend;
pub mod extra;
pub mod errors;
pub mod tree;
pub mod sol;
pub mod log;




use controller::Controller;
use delegate::Delegation;
use rule::RuleAccumulator;
use spend::{SpendRequest, delegation_account_size};


declare_id!("TRSY7YgS3tcDoi6ZgTp2MmPJpXHyCVrGaFhL7HLdQc9");
// devnet
//declare_id!("Hq9C7gEEEYFACvGVoGZzPzZ5uakj5ccYDyfdqM6m2g2n");
// localnet#[cfg(feature = "localnet")]
//declare_id!("8xezzAjRLMz7ysry3ci7grG71dk11UjpmKFt1wvo8e9s");

#[program]
pub mod safejar {
    use anchor_lang::solana_program::entrypoint::ProgramResult;

    use super::*;

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn create_controller(ctx: Context<CreateController>) -> ProgramResult{
        nplog!("np initialize - 1");
        ctx.accounts.process(&ctx.bumps.controller)?;
        Ok(())
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn close_controller(ctx: Context<CloseController>) -> ProgramResult{
        ////msg!("close controller - 1");
        ctx.accounts.process()?;
        Ok(())
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn close_controller_vault(ctx: Context<CloseControllerVault>) ->ProgramResult{
        ////msg!("close controller vault - 1");
        ctx.accounts.process()?;
        Ok(())
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn transfer_to_controller(ctx: Context<TransferToController>,amount: u64)-> ProgramResult{
        return ctx.accounts.process(amount);
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn transfer_to_delegation(ctx: Context<TransferToDelegation>,amount: u64)-> ProgramResult{
        return ctx.accounts.process(amount);
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn create_rule_accumulator(
        ctx: Context<CreateRuleAccumulator>,
        tree: Vec<u8>,
    )->ProgramResult{
        return ctx.accounts.process(tree);
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn rule_add_rate_limiter(
        ctx: Context<RuleAddRateLimiter>,
        max_spend: u64, delta_slot: u64,
    )->ProgramResult{
        return ctx.accounts.process(max_spend,delta_slot);
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn rule_add_authorization_constraint(
        ctx: Context<RuleAddAuthorizationConstraint>,
    )->ProgramResult{
        return ctx.accounts.process();
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn rule_add_program_constraint(
        ctx: Context<RuleAddProgramConstraint>,
    )->ProgramResult{
        return ctx.accounts.process();
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn rule_add_balance_constraint(
        ctx: Context<RuleAddBalanceConstraint>,
        max_balance: u64,
    )->ProgramResult{
        return ctx.accounts.process(max_balance);
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn rule_add_sweep(
        ctx: Context<RuleAddSweep>,
        min_balance: u64,
    )->ProgramResult{
        return ctx.accounts.process(min_balance);
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn rule_add_sweep_ata(
        ctx: Context<RuleAddSweepATA>,
        min_balance: u64,
    )->ProgramResult{
        return ctx.accounts.process(min_balance);
    }


    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn delegate(ctx: Context<Delegate>,max_spend_state: u8)->ProgramResult{
        nplog!("max spend state - {}",max_spend_state);
        msg!("___mss {}",max_spend_state);
        return ctx.accounts.process(ctx.bumps.delegation,max_spend_state);
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn approve_delegation(ctx: Context<ApproveDelegation>)->ProgramResult{
        return ctx.accounts.process();
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn reject_delegation(ctx: Context<RejectDelegation>)->ProgramResult{
        return ctx.accounts.process();
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    //#[inline(never)]
    pub fn create_spend_request_direct(
        ctx: Context<CreateSpendRequestDirect>,
        amount: u64,
        tree: Vec<u8>,
    )->ProgramResult{
        nplog!("create - 000.000 - 1");
        return ctx.accounts.process(amount,tree);
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn complete_spend_request_direct(ctx: Context<CompleteSpendRequestDirect>)->ProgramResult{
        return ctx.accounts.process();
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn close_delegation(ctx: Context<CloseDelegation>)->ProgramResult{
        return ctx.accounts.process();
    }


    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn rule_process_rate_limiter(
        ctx: Context<SpendProcessRateLimiter>,
        max_spend: u64, delta_slot: u64,
    )->ProgramResult{
        return ctx.accounts.process(max_spend,delta_slot);
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn rule_process_authorization_constraint(
        ctx: Context<SpendProcessAuthorizationConstraint>,
    )->ProgramResult{
        
        //ctx.accounts.authorizer.is_signer;
        
        return ctx.accounts.process();
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn rule_process_non_authorization_constraint(
        ctx: Context<SpendProcessAuthorizationConstraintNoSigner>,
    )->ProgramResult{
        
        return ctx.accounts.process();
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn rule_process_program_constraint(
        ctx: Context<SpendProcessProgramConstraint>,
    )->ProgramResult{
        return ctx.accounts.process();
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn rule_process_balance_constraint(
        ctx: Context<SpendProcessBalanceConstraint>,
        max_balance: u64,
    )->ProgramResult{
        return ctx.accounts.process(max_balance);
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn rule_process_sweep(
        ctx: Context<SpendProcessSweep>,
        min_balance: u64,
    )->ProgramResult{
        return ctx.accounts.process(min_balance);
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn consolidate_vault(
        ctx: Context<ConsolidateVault>,
    )->ProgramResult{
        return ctx.accounts.process();
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn transfer_sol(
        ctx: Context<RentManagerTransferSOL>,
    )->ProgramResult{
        //msg!("transfer - 1");
        return ctx.accounts.process();
    }


    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn subsidize(
        ctx: Context<RentManagerSubsidize>,
        amount: u64,
    )->ProgramResult{
        //msg!("subsidize - 1");
        return ctx.accounts.process(amount);
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn initialize_destination(
        ctx: Context<InitializeDestination>,
    )->ProgramResult{
        //msg!("initializing account");
        return ctx.accounts.process();
    }
}


#[derive(Accounts)]
#[instruction()]
pub struct CreateController<'info>{
    #[account(
        init,
        payer = payer,
        seeds=[PROGRAM_CONTROLLER_SEED,owner.key().as_ref()],
        bump,
        space=8 + std::mem::size_of::<Controller>(),
    )]
    pub controller: Account<'info,Controller>,


    #[account()]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    //pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
#[instruction()]
pub struct CloseController<'info>{
    #[account(
        mut,
        close = owner,
        seeds=[PROGRAM_CONTROLLER_SEED,owner.key().as_ref()],
        bump=controller.bump,
        constraint=controller.delegation_count==0,
        constraint=controller.owner==owner.key(),
    )]
    pub controller: Account<'info,Controller>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction()]
pub struct CloseControllerVault<'info>{
    #[account(
        seeds=[PROGRAM_CONTROLLER_SEED,owner.key().as_ref()],
        bump=controller.bump,
        constraint=controller.owner==owner.key(),
    )]
    pub controller: Account<'info,Controller>,

    #[account(
        mut,
        constraint=controller_vault.owner==controller.key(),
        constraint=controller_vault.mint==mint.key(),
    )]
    pub controller_vault: Account<'info,TokenAccount>,

    #[account(
        init_if_needed,
        payer = fee_payer,
        associated_token::mint = mint,
        associated_token::authority = owner,
    )]
    pub owner_vault: Account<'info,TokenAccount>,

    pub owner: Signer<'info>,
    #[account(mut)]
    pub fee_payer: Signer<'info>,

    pub mint: Account<'info,Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info,AssociatedToken>,
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct TransferToDelegation<'info>{
    #[account(
        seeds=[PROGRAM_CONTROLLER_SEED,controller.owner.as_ref()],
        bump=controller.bump,
        constraint=controller.owner==owner.key(),
    )]
    pub controller: Account<'info,Controller>,

    #[account(
        mut,
        constraint=controller_vault.mint==mint.key(),
        constraint=controller_vault.owner==controller.key(),
    )]
    pub controller_vault: Account<'info,TokenAccount>,

    #[account(
        //mut,
        seeds=[PROGRAM_DELEGATION_SEED,controller.key().as_ref(),delegation.rule_set_hash.as_ref()],
        bump=delegation.bump,
        constraint=delegation.controller==controller.key(),
    )]
    pub delegation: Box<Account<'info,Delegation>>,

    #[account(
        init_if_needed,
        payer = fee_payer,
        associated_token::mint = mint,
        associated_token::authority = delegation,
    )]
    pub delegation_vault: Account<'info,TokenAccount>,

    #[account()]
    pub mint: Account<'info,Mint>,

    pub owner: Signer<'info>,
    #[account(mut)]
    pub fee_payer: Signer<'info>,

    //pub rent: Sysvar<'info,Rent>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info,AssociatedToken>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct TransferToController<'info>{
    #[account(
        mut,
        seeds=[PROGRAM_CONTROLLER_SEED,controller.owner.as_ref()],
        bump=controller.bump,
        constraint=controller.owner==owner.key(),
    )]
    pub controller: Account<'info,Controller>,

    #[account(
        init_if_needed,
        payer = fee_payer,
        associated_token::mint = mint,
        associated_token::authority = controller,
    )]
    pub controller_vault: Account<'info,TokenAccount>,

    #[account(
        //mut,
        seeds=[PROGRAM_DELEGATION_SEED,controller.key().as_ref(),delegation.rule_set_hash.as_ref()],
        bump=delegation.bump,
        constraint=delegation.controller==controller.key(),
    )]
    pub delegation: Box<Account<'info,Delegation>>,

    #[account(
        mut,
        constraint=delegation_vault.mint==mint.key(),
        constraint=delegation_vault.owner==delegation.key(),
    )]
    pub delegation_vault: Account<'info,TokenAccount>,

    #[account()]
    pub mint: Account<'info,Mint>,

    pub owner: Signer<'info>,
    #[account(mut)]
    pub fee_payer: Signer<'info>,

    //pub rent: Sysvar<'info,Rent>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info,AssociatedToken>,
    pub system_program: Program<'info, System>,
}



#[derive(Accounts)]
#[instruction(tree: Box<Vec<u8>>)]
pub struct CreateRuleAccumulator<'info>{
    #[account(
        seeds=[PROGRAM_CONTROLLER_SEED,controller.owner.as_ref()],
        bump=controller.bump,
        constraint=controller.owner==owner.key(),
    )]
    pub controller: Account<'info,Controller>,

    /// CHECK: do not check; controller general funds pays rent
    #[account(
        init,
        payer = linker,
        space=8+std::mem::size_of::<RuleAccumulator>(),
    )]
    pub accumulator: Account<'info,RuleAccumulator>,

    
    pub owner: Signer<'info>,
    #[account(mut)]
    pub linker: Signer<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    
}


#[derive(Accounts)]
#[instruction(max_spend: u8, delta_slot: u64)]
pub struct RuleAddRateLimiter<'info>{
    #[account(
        seeds=[PROGRAM_CONTROLLER_SEED,controller.owner.as_ref()],
        bump=controller.bump,
        constraint=controller.owner==owner.key(),
    )]
    pub controller: Account<'info,Controller>,

    #[account(
        mut,
        constraint=accumulator.controller==controller.key(),
        // run arg constraint check in RateLimiter struct
    )]
    pub accumulator: Box<Account<'info,RuleAccumulator>>,

    pub owner: Signer<'info>,

    pub mint: Account<'info,Mint>,

}


#[derive(Accounts)]
#[instruction()]
pub struct RuleAddAuthorizationConstraint<'info>{
    #[account(
        seeds=[PROGRAM_CONTROLLER_SEED,controller.owner.as_ref()],
        bump=controller.bump,
        constraint=controller.owner==owner.key(),
    )]
    pub controller: Account<'info,Controller>,

    #[account(
        mut,
        constraint=accumulator.controller==controller.key(),
        // run arg constraint check in RateLimiter struct
    )]
    pub accumulator: Box<Account<'info,RuleAccumulator>>,

    /// CHECK: we just need the public key from the authorizer
    // the entity that will make spending decisions
    pub authorizer: AccountInfo<'info>,

    #[account()]
    pub owner: Signer<'info>,

}

#[derive(Accounts)]
#[instruction()]
pub struct RuleAddProgramConstraint<'info>{
    #[account(
        seeds=[PROGRAM_CONTROLLER_SEED,controller.owner.as_ref()],
        bump=controller.bump,
        constraint=controller.owner==owner.key(),
    )]
    pub controller: Account<'info,Controller>,

    #[account(
        mut,
        constraint=accumulator.controller==controller.key(),
        // run arg constraint check in RateLimiter struct
    )]
    pub accumulator: Box<Account<'info,RuleAccumulator>>,

    /// CHECK: we just need to check if this is executable and the public key
    // the owner of the owner of the token account receiving funds
    #[account(
        constraint=program.executable
    )]
    pub program: AccountInfo<'info>,

    #[account()]
    pub owner: Signer<'info>,

}


#[derive(Accounts)]
#[instruction(max_balance: u64)]
pub struct RuleAddBalanceConstraint<'info>{
    #[account(
        seeds=[PROGRAM_CONTROLLER_SEED,controller.owner.as_ref()],
        bump=controller.bump,
        constraint=controller.owner==owner.key(),
    )]
    pub controller: Account<'info,Controller>,

    #[account(
        mut,
        constraint=accumulator.controller==controller.key(),
        // run arg constraint check in RateLimiter struct
    )]
    pub accumulator: Box<Account<'info,RuleAccumulator>>,

    pub owner: Signer<'info>,

    pub mint: Account<'info,Mint>,
}

#[derive(Accounts)]
#[instruction(min_balance: u64)]
pub struct RuleAddSweep<'info>{
    #[account(
        seeds=[PROGRAM_CONTROLLER_SEED,controller.owner.as_ref()],
        bump=controller.bump,
        constraint=controller.owner==owner.key(),
    )]
    pub controller: Account<'info,Controller>,

    #[account(
        mut,
        constraint=accumulator.controller==controller.key(),
        // run arg constraint check in RateLimiter struct
    )]
    pub accumulator: Box<Account<'info,RuleAccumulator>>,

    pub owner: Signer<'info>,

    pub destination: Account<'info,TokenAccount>,

}


#[derive(Accounts)]
#[instruction(min_balance: u64)]
pub struct RuleAddSweepATA<'info>{
    #[account(
        seeds=[PROGRAM_CONTROLLER_SEED,controller.owner.as_ref()],
        bump=controller.bump,
        constraint=controller.owner==owner.key(),
    )]
    pub controller: Account<'info,Controller>,

    #[account(
        mut,
        constraint=accumulator.controller==controller.key(),
        // run arg constraint check in RateLimiter struct
    )]
    pub accumulator: Box<Account<'info,RuleAccumulator>>,

    pub owner: Signer<'info>,

    #[account(mut)]
    pub fee_payer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = fee_payer,
        associated_token::mint = mint, 
        associated_token::authority = destination_owner,
    )]
    pub destination: Account<'info,TokenAccount>,

    /// CHECK: we only need the pubkey for destination
    pub destination_owner: UncheckedAccount<'info>,

    pub mint: Account<'info,Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info,Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,

}

#[derive(Accounts)]
#[instruction(max_spend_state: u8)]
pub struct Delegate<'info>{
    #[account(
        mut,
        seeds=[PROGRAM_CONTROLLER_SEED,controller.owner.as_ref()],
        bump=controller.bump,
        constraint=controller.owner==owner.key(),
        constraint=controller.key()==accumulator.controller,
    )]
    pub controller: Account<'info,Controller>,

    #[account(
        init,
        payer = linker,
        seeds=[PROGRAM_DELEGATION_SEED,controller.key().as_ref(),accumulator.hash.as_ref()],
        bump,
        space=delegation_account_size(max_spend_state),
        constraint=0<max_spend_state,
    )]
    pub delegation: Box<Account<'info,Delegation>>,

    #[account(
        mut,
        close = linker,
        signer,
    )]
    pub accumulator: Box<Account<'info,RuleAccumulator>>,

    #[account(mut)]
    pub linker: Signer<'info>,
    pub owner: Signer<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}


#[derive(Accounts)]
#[instruction()]
pub struct ApproveDelegation<'info>{
    #[account(
        mut,
        seeds=[PROGRAM_CONTROLLER_SEED,owner.key().as_ref()],
        bump=controller.bump,
        constraint=controller.owner==owner.key(),
    )]
    pub controller: Account<'info,Controller>,

    #[account(
        seeds=[PROGRAM_DELEGATION_SEED,controller.key().as_ref(),delegation.rule_set_hash.as_ref()],
        bump=delegation.bump,
        constraint=0<delegation.requested_slot
    )]
    pub delegation: Box<Account<'info,Delegation>>,

    pub owner: Signer<'info>,
}

#[derive(Accounts)]
#[instruction()]
pub struct RejectDelegation<'info>{
    #[account(
        mut,
        seeds=[PROGRAM_CONTROLLER_SEED,controller.owner.as_ref()],
        bump=controller.bump,
        constraint=controller.key()==delegation.controller,
    )]
    pub controller: Account<'info,Controller>,

    #[account(
        mut,
        close = controller,
        seeds=[PROGRAM_DELEGATION_SEED,controller.key().as_ref(),delegation.rule_set_hash.as_ref()],
        bump=delegation.bump,
        constraint=0<delegation.requested_slot,
        constraint=delegation.requested_slot + 1000 < clock.slot
    )]
    pub delegation: Box<Account<'info,Delegation>>,

    #[account(mut)]
    pub rejector: Signer<'info>,

    pub clock: Sysvar<'info, Clock>,
}

// TODO: write CloseTokenAccount

#[derive(Accounts)]
#[instruction()]
pub struct CloseDelegation<'info>{
    #[account(
        mut,
        seeds=[PROGRAM_CONTROLLER_SEED,controller.owner.as_ref()],
        bump=controller.bump,
        constraint=controller.owner==owner.key(),
    )]
    pub controller: Account<'info,Controller>,

    #[account(
        mut,
        close = linker,
        seeds=[PROGRAM_DELEGATION_SEED,controller.key().as_ref(),delegation.rule_set_hash.as_ref()],
        bump=delegation.bump,
    )]
    pub delegation: Box<Account<'info,Delegation>>,

    pub owner: Signer<'info>,
    #[account(mut)]
    pub linker: Signer<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}




#[derive(Accounts)]
#[instruction(amount: u64,tree: Vec<u8>)]
pub struct CreateSpendRequestDirect<'info>{

    #[account(
        //constraint=delegation.requested_slot==0,
    )]
    pub delegation: Box<Account<'info,Delegation>>,

    ///CHECK: skip check
    #[account(
        init,
        signer,
        payer = linker,
        space=8+std::mem::size_of::<SpendRequest>()+8+tree.len()+300,
    )]
    pub request: Account<'info,SpendRequest>,

    // SOURCE OF FUNDS
    // we only spend from an ATA account.  This way, we can enforce balance constraints.
    // TODO: create instruction to consolidate token account balances of the same mint, and close those token accounts
    #[account(
        mut,
        constraint=delegation_vault.owner==delegation.key(),
        constraint=delegation_vault.mint==destination_vault.mint,
        constraint=is_ata(&delegation_vault.key(),&delegation.key(),&delegation_vault.mint),
    )]
    pub delegation_vault: Box<Account<'info,TokenAccount>>,

    #[account(
        init_if_needed,
        payer = linker,
        associated_token::mint = mint,
        associated_token::authority = destination_owner,
    )]
    pub destination_vault: Box<Account<'info,TokenAccount>>,

    /// CHECK: we only need the pubkey for destination
    pub destination_owner: SystemAccount<'info>,

    pub mint: Account<'info,Mint>,

    #[account(mut)]
    pub linker: Signer<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info,AssociatedToken>,
}

#[derive(Accounts)]
#[instruction(max_spend: u64, delta_slot: u64)]
pub struct SpendProcessRateLimiter<'info>{
    #[account(mut)]
    pub request: Box<Account<'info,SpendRequest>>,

    // this is the mint of the rule, not the mint of the spend request
    pub required_mint: Account<'info,Mint>,

    #[account(
        constraint=request.context.linker==linker.key(),
    )]
    pub linker: Signer<'info>,
}


#[derive(Accounts)]
#[instruction()]
pub struct SpendProcessAuthorizationConstraint<'info>{
    #[account(mut)]
    pub request: Box<Account<'info,SpendRequest>>,

    // only input the authorizer if there is one
    /// CHECK: we only need the public key
    pub required_authorizer: AccountInfo<'info>,

    #[account(
        constraint=required_authorizer.key()==authorizer.key(),
    )]
    pub authorizer: Signer<'info>,

    #[account(
        constraint=request.context.linker==linker.key(),
    )]
    pub linker: Signer<'info>,
}

#[derive(Accounts)]
#[instruction()]
pub struct SpendProcessAuthorizationConstraintNoSigner<'info>{
    #[account(mut)]
    pub request: Box<Account<'info,SpendRequest>>,

    // only input the authorizer if there is one
    /// CHECK: we only need the public key
    pub required_authorizer: AccountInfo<'info>,

    #[account(
        constraint=request.context.linker==linker.key(),
    )]
    pub linker: Signer<'info>,
}


#[derive(Accounts)]
#[instruction()]
pub struct SpendProcessProgramConstraint<'info>{
    #[account(mut)]
    pub request: Box<Account<'info,SpendRequest>>,

    // compare this program to the one in request.state
    /// CHECK: we only need to know if this is executable and we need the public key
    #[account(
        constraint=required_program.executable
    )]
    pub required_program: AccountInfo<'info>,

    #[account(
        constraint=request.context.linker==linker.key(),
    )]
    pub linker: Signer<'info>,
}



// balance constraints only apply to a single token account, NOT to a mint
// delegation accounts can only spend from ATA token accounts.
// this is how we indirectly enforce a balance constraint on a mint.
#[derive(Accounts)]
#[instruction(max_balance: u64)]
pub struct SpendProcessBalanceConstraint<'info>{
    #[account(
        mut,
        constraint=request.context.source_vault==delegation_vault.key()
    )]
    pub request: Box<Account<'info,SpendRequest>>,

    pub delegation_vault: Account<'info,TokenAccount>,

    #[account(
        constraint=request.context.linker==linker.key(),
    )]
    pub linker: Signer<'info>,
}




#[derive(Accounts)]
#[instruction(min_balance: u64)]
pub struct SpendProcessSweep<'info>{
    #[account(mut)]
    pub request: Box<Account<'info,SpendRequest>>,

    // compare this to what is in the spend request
    pub required_destination: Account<'info,TokenAccount>,
    
    #[account(
        constraint=request.context.linker==linker.key(),
    )]
    pub linker: Signer<'info>,
}


#[derive(Accounts)]
#[instruction()]
pub struct CompleteSpendRequestDirect<'info>{

    #[account(
        mut,
        close=linker,
        constraint=log_me("csrd - 1"),
        constraint=request.delegation==delegation.key(),
        constraint=log_me("csrd - 2"),
        constraint=request.context.source_vault==delegation_vault.key(),
        constraint=log_me("csrd - 3"),
        constraint=request.context.destination_vault==destination_vault.key(),
        constraint=log_me("csrd - 4"),
        constraint=hash_is_equal(&request.hash,&delegation.rule_set_hash),
        constraint=log_me("csrd - 5"),
    )]
    pub request: Box<Account<'info,SpendRequest>>,

    #[account(
        mut,
        seeds=[PROGRAM_DELEGATION_SEED,delegation.controller.as_ref(),delegation.rule_set_hash.as_ref()],
        bump=delegation.bump,
        constraint=delegation.key()==delegation_vault.owner,
    )]
    pub delegation: Box<Account<'info,Delegation>>,

    // SOURCE OF FUNDS! (delegation and destination words look very similar, be careful!)
    #[account(
        mut,
    )]
    pub delegation_vault: Account<'info,TokenAccount>,

    // DESTINATION OF FUNDS
    #[account(
        mut,
    )]
    pub destination_vault: Account<'info,TokenAccount>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    #[account(
        mut,
        constraint=request.context.linker==linker.key(),
    )]
    pub linker: Signer<'info>,


}

fn is_ata<'a>(vault: &Pubkey,owner: &'a Pubkey, mint: &'a Pubkey)->bool{
    let token_program_id = anchor_spl::token::spl_token::ID;
    let ata_program_id = anchor_spl::associated_token::ID;
    let seeds = [
        owner.as_ref(),
        token_program_id.as_ref(),
        mint.as_ref(),
    ];
    
    let (vault_check,_bump_check)=Pubkey::find_program_address(&seeds, &ata_program_id);
    
    return vault.cmp(&vault_check)==Ordering::Equal;
}


#[derive(Accounts)]
#[instruction()]
pub struct ConsolidateVault<'info>{
    #[account(
        seeds=[PROGRAM_DELEGATION_SEED,delegation.controller.as_ref(),delegation.rule_set_hash.as_ref()],
        bump=delegation.bump,
        constraint=delegation.requested_slot==0,
    )]
    pub delegation: Box<Account<'info,Delegation>>,

    #[account(
        mut,
        constraint=rent_sol_vault.owner==delegation.key(),
        constraint=rent_sol_vault.mint==WSOL,
    )]
    pub rent_sol_vault: Box<Account<'info,TokenAccount>>,

    // SOURCE OF FUNDS! (delegation and destination words look very similar, be careful!)
    #[account(
        mut,
        constraint=is_ata(&ata_vault.key(),&delegation.key(),&mint.key()),
        constraint=ata_vault.mint==mint.key(),
        constraint=ata_vault.owner==delegation.key(),
    )]
    pub ata_vault: Box<Account<'info,TokenAccount>>,

    #[account(
        mut,
        constraint=ata_vault.key()!=vault_1.key(),
        constraint=ata_vault.mint==vault_1.mint,
        constraint=ata_vault.owner==vault_1.owner,
    )]
    pub vault_1: Box<Account<'info,TokenAccount>>,

    #[account(
        mut,
        constraint=ata_vault.key()!=vault_2.key(),
        constraint=ata_vault.mint==vault_2.mint,
        constraint=ata_vault.owner==vault_2.owner,
    )]
    pub vault_2: Option<Box<Account<'info,TokenAccount>>>,

    #[account(
        mut,
        constraint=ata_vault.key()!=vault_3.key(),
        constraint=ata_vault.mint==vault_3.mint,
        constraint=ata_vault.owner==vault_3.owner,
    )]
    pub vault_3: Option<Box<Account<'info,TokenAccount>>>,

    #[account(
        mut,
        constraint=ata_vault.key()!=vault_4.key(),
        constraint=ata_vault.mint==vault_4.mint,
        constraint=ata_vault.owner==vault_4.owner,
    )]
    pub vault_4: Option<Box<Account<'info,TokenAccount>>>,

    #[account(
        mut,
        constraint=ata_vault.key()!=vault_5.key(),
        constraint=ata_vault.mint==vault_5.mint,
        constraint=ata_vault.owner==vault_5.owner,
    )]
    pub vault_5: Option<Box<Account<'info,TokenAccount>>>,

    pub token_program: Program<'info, Token>,
    pub mint: Account<'info,Mint>,
}



pub const PROGRAM_CONTROLLER_SEED: &[u8] = b"controller";
pub const PROGRAM_DELEGATION_SEED: &[u8] = b"delegation";

fn log_me(_s: &str)->bool{
    //msg!("{}",s);
    return true
}


pub(crate) fn hash_is_equal(a: &[u8],b: &[u8])->bool{
    if a.len() !=b.len(){
        ////msg!("bad length with a={} and b={}",a.len(),b.len());
        //msg!("hash is not equal");
        return false
    }
    for i in 0..a.len(){
        ////msg!("i={} with a={} and b={}",i,a[i],b[i]);
        if a[i]!=b[i]{
            //msg!("hash is not equal");
            return false
        }
    }
    //msg!("hash is equal");
    return true
}



#[derive(Accounts)]
#[instruction()]
pub struct RentManagerTransferSOL<'info>{
    #[account(
        mut,
        constraint=source_vault.mint==WSOL,
        constraint=source_vault.owner==source_owner.key(),
    )]
    pub source_vault: Account<'info,TokenAccount>,

    pub source_owner: Signer<'info>,

    /// CHECK: this holds SOL that we will transfer to the target and change_sol; this account will be closed
    #[account(mut)]
    pub tmp_sys: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}


#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct RentManagerSubsidize<'info>{

    /// CHECK: we are just using this account as a reference in a constraint
    pub source_owner: AccountInfo<'info>,

    /// CHECK: this holds SOL that we will transfer to the target and change_sol; this account will be closed
    #[account(
        mut,
        signer,
    )]
    pub tmp_sys: AccountInfo<'info>,

    #[account(
        init_if_needed,
        payer = tmp_sys,
        token::mint = mint, 
        token::authority = source_owner
    )]
    pub change_sol: Account<'info,TokenAccount>,

    /// CHECK: send "amount" SOL to here to pay the target's rent
    #[account(mut)]
    pub target: AccountInfo<'info>,

    pub mint: Account<'info,Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}



#[derive(Accounts)]
#[instruction()]
pub struct InitializeDestination<'info>{


    ///CHECK: we just need the owner
    pub destination_owner: AccountInfo<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint=mint,
        associated_token::authority=destination_owner,
    )]
    pub destination_vault: Account<'info,TokenAccount>,

    pub mint: Account<'info,Mint>,

    pub associated_token_program: Program<'info,AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,

    #[account(mut)]
    pub payer: Signer<'info>,
}
impl<'info> InitializeDestination<'info>{
    pub fn process(&self)->ProgramResult{
        return Ok(())
    }
}


