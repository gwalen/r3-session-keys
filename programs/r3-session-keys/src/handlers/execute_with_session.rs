use anchor_lang::prelude::*;

use crate::instructions::execute_with_session::ExecuteWithSession;
use crate::{
    state::{program_config::ProgramConfig, session::Session, user_smart_wallet::UserSmartWallet},
    utils::{common::read_array_element, errors::DappError},
};
use anchor_lang::solana_program::{instruction::Instruction, program::invoke_signed};

pub fn handle<'info>(
    ctx: Context<ExecuteWithSession<'info>>,
    instruction_data: Vec<u8>,
) -> Result<()> {
    validate_session_constraints(&ctx)?;

    validate_target_instruction(&ctx, &instruction_data)?;

    execute_target_instruction(&ctx, &instruction_data)?;

    // TODO: if needed, post execution checks goes here (for example SOL or token from smart wallet used)

    // TODO: emit event

    Ok(())
}

fn execute_target_instruction<'info>(
    ctx: &Context<ExecuteWithSession<'info>>,
    instruction_data: &[u8],
) -> Result<()> {
    let mut account_metas = Vec::<AccountMeta>::new();
    let mut account_infos = Vec::<AccountInfo>::new();

    for i in 0..ctx.remaining_accounts.len() {
        if ctx.remaining_accounts[i].key() == ctx.accounts.user_smart_wallet.key() {
            // it is user_smart_wallet account so we must mark it as signer and readonly
            account_metas.push(AccountMeta::new_readonly(ctx.remaining_accounts[i].key(), true));
            account_infos.push(ctx.remaining_accounts[i].to_account_info());
        } else {
            account_metas.push(create_account_meta(&ctx.remaining_accounts[i]));
            account_infos.push(ctx.remaining_accounts[i].to_account_info());
        }
    }

    let instruction = Instruction {
        program_id: ctx.accounts.target_program.key(),
        accounts: account_metas,
        // data: instruction_data.clone(),
        data: instruction_data.to_vec(),
    };

    let smart_wallet_signer_seeds: &[&[&[u8]]] = &[&[
        UserSmartWallet::SEED_PREFIX,
        &ctx.accounts.user_smart_wallet.smart_wallet_owner.key().to_bytes(),
        &[ctx.accounts.user_smart_wallet.bump],
    ]];

    invoke_signed(&instruction, &account_infos, smart_wallet_signer_seeds)?;

    Ok(())
}

fn validate_session_constraints<'info>(ctx: &Context<ExecuteWithSession<'info>>) -> Result<()> {
    validate_session_expiration(ctx)?;
    Ok(())
}

fn validate_session_expiration<'info>(ctx: &Context<ExecuteWithSession<'info>>) -> Result<()> {
    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp < ctx.accounts.session.expires_at,
        DappError::SessionExpired
    );
    Ok(())
}

fn validate_target_instruction<'info>(
    ctx: &Context<ExecuteWithSession<'info>>,
    instruction_data: &Vec<u8>,
) -> Result<()> {
    // Instruction data must be at least as long as the ix discriminator size
    require!(
        instruction_data.len() >= ctx.accounts.session.discriminator_size as usize,
        DappError::EmptyInstructionData
    );
    // we do not in proxied call to make call to it self (solana reentrancy would prevent it but we add it for checks completeness)
    require!(
        ctx.accounts.target_program.key() != crate::ID,
        DappError::NotAllowedToCallSmartWalletProgram
    );

    validate_discriminator(ctx, instruction_data)?;
    validate_instruction_accounts(ctx)?;

    Ok(())
}

fn validate_instruction_accounts<'info>(ctx: &Context<ExecuteWithSession<'info>>) -> Result<()> {
    // It not allowed for remaining accounts to contain program_config or session
    // because malicious caller could override the privileges to this accounts by making
    // them writable and modify program_config, session
    require!(
        !contains_account(ctx.remaining_accounts, &ctx.accounts.program_config.key()),
        DappError::RemainingAccountsContainsProgramConfig
    );
    require!(
        !contains_account(ctx.remaining_accounts, &ctx.accounts.session.key()),
        DappError::RemainingAccountsContainsSession
    );
    // It not allowed for remaining accounts to contain session_key, it is for session use only
    require!(
        !contains_account(ctx.remaining_accounts, &ctx.accounts.session_key.key()),
        DappError::RemainingAccountsContainsSessionKey
    );

    // for smart_wallet account it is more complex as caller will have to pass it account list
    // because it will be a singer for invoke_signed (tx proxy)
    // but it can not be writable and must be only one
    let smart_wallet_accounts_found = find_all_accounts(ctx.remaining_accounts, &ctx.accounts.user_smart_wallet.key());
    require!(
        smart_wallet_accounts_found.len() == 1,
        DappError::MultipleUserSmartWalletAccounts
    );
    require!(
        !smart_wallet_accounts_found[0].is_writable,
        DappError::UserSmartWalletAccountIsWritable
    );

    // TODO: double check if any other accounts should be checked here (- imo no)

    Ok(())
}

fn validate_discriminator<'info>(
    ctx: &Context<ExecuteWithSession<'info>>,
    instruction_data: &Vec<u8>,
) -> Result<()> {
    let session = &ctx.accounts.session;

    let disc_size = session.discriminator_size as usize;
    let discriminators = session.parse_discriminators();
    let mut is_allowed = false;
    for disc in &discriminators {
        if instruction_data[..disc_size] == *disc {
            is_allowed = true;
            break;
        }
    }
    require!(is_allowed, DappError::NotAllowedInstructionDiscriminator);

    Ok(())
}

fn contains_account<'info>(
    remaining_accounts: &[AccountInfo<'info>],
    account_to_check: &Pubkey,
) -> bool {
    for account in remaining_accounts {
        if account.key() == *account_to_check {
            return true;
        }
    }
    false
}

fn find_all_accounts<'a, 'info>(
    remaining_accounts: &'a [AccountInfo<'info>],
    account_to_find: &Pubkey,
) -> Vec<&'a AccountInfo<'info>> {
    let mut found = Vec::new();
    for account in remaining_accounts {
        if account.key() == *account_to_find {
            found.push(account);
        }
    }
    found
}

fn create_account_meta(target_account: &AccountInfo<'_>) -> AccountMeta {
    let is_signer = target_account.is_signer;
    if target_account.is_writable {
        AccountMeta::new(target_account.key(), is_signer)
    } else {
        AccountMeta::new_readonly(target_account.key(), is_signer)
    }
}
