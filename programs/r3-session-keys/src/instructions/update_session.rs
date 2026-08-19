use anchor_lang::prelude::*;

use crate::{
    state::{
        program_config::{ProgramConfig, ProgramStatus},
        session::{Session, SessionStatus},
        user_smart_wallet::UserSmartWallet,
    },
    utils::errors::DappError,
};

#[derive(Accounts)]
#[instruction(_session_key: Pubkey)]
pub struct UpdateSession<'info> {
    pub session_executor: Signer<'info>,

    #[account(
        seeds = [ProgramConfig::SEED_PREFIX],
        constraint = program_config.status == ProgramStatus::Active @ DappError::ProgramPaused,
        bump = program_config.bump,
    )]
    pub program_config: Account<'info, ProgramConfig>,

    pub user_smart_wallet: Account<'info, UserSmartWallet>,

    #[account(
        mut,
        seeds = [Session::SEED_PREFIX, user_smart_wallet.key().as_ref(), _session_key.key().as_ref()],
        constraint = session.session_executor == session_executor.key() @ DappError::UnauthorizedSessionExecutor,
        constraint = session.status != SessionStatus::Revoked @ DappError::InvalidSessionStatus,
        bump = session.bump
    )]
    pub session: Box<Account<'info, Session>>,
}
