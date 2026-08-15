use anchor_lang::prelude::*;

use crate::{
    state::session::Session,
    utils::errors::DappError,
    state::{program_config::ProgramConfig, user_smart_wallet::UserSmartWallet},
    state::program_config::ProgramStatus,
};


#[derive(Accounts)]
#[instruction(session_key: Pubkey, smart_wallet: Pubkey)]
pub struct CreateSession<'info> {
    #[account(mut)]
    pub session_owner: Signer<'info>,

    #[account(
        constraint = program_config.status == ProgramStatus::Active @ DappError::ProgramPaused
    )]
    pub program_config: Account<'info, ProgramConfig>,

    #[account(
        address = smart_wallet @ DappError::UserSmartWalletNotFound
    )]
    pub user_smart_wallet: Account<'info, UserSmartWallet>,

    // seeds: [b"session", user_smart_wallet.to_bytes().as_ref(), session_key.to_bytes().as_ref()]
    #[account(
        init,
        payer = session_owner,
        space = 8 + Session::INIT_SPACE,
        seeds = [Session::SEED_PREFIX, user_smart_wallet.key().as_ref(), session_key.key().as_ref()],
        bump
    )]
    pub session: Account<'info, Session>,

    pub system_program: Program<'info, System>,
}