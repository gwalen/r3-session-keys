use anchor_lang::prelude::*;

use crate::{
    state::session::Session,
    utils::errors::DappError,
    state::{program_config::ProgramConfig, user_smart_wallet::UserSmartWallet},
    state::program_config::ProgramStatus,
};


#[derive(Accounts)]
#[instruction(_session_key: Pubkey, _smart_wallet: Pubkey)]
pub struct RevokeSession<'info> {
    pub session_owner: Signer<'info>,

    #[account(
        constraint = program_config.status == ProgramStatus::Active @ DappError::ProgramPaused
    )]
    pub program_config: Account<'info, ProgramConfig>,

    #[account(
        address = _smart_wallet @ DappError::UserSmartWalletNotFound
    )]
    pub user_smart_wallet: Account<'info, UserSmartWallet>,

    #[account(
        mut,
        seeds = [Session::SEED_PREFIX, user_smart_wallet.key().as_ref(), _session_key.key().as_ref()],
        bump = session.bump
    )]
    pub session: Account<'info, Session>,
}
