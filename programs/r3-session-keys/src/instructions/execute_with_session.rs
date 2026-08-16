use anchor_lang::prelude::*;

use crate::{
    state::{program_config::ProgramConfig, session::Session, user_smart_wallet::UserSmartWallet},
    utils::errors::DappError,
};
use crate::state::{program_config::ProgramStatus, session::SessionStatus};

#[derive(Accounts)]
pub struct ExecuteWithSession<'info> {
    // Bot / external caller this session was created for. Distinct from user_smart_wallet.smart_wallet_owner.
    pub session_executor: Signer<'info>,

    // Ephemeral key proving the caller holds this session
    pub session_key: Signer<'info>,

    #[account(
        seeds = [ProgramConfig::SEED_PREFIX],
        constraint = program_config.status == ProgramStatus::Active @ DappError::ProgramPaused,
        bump = program_config.bump,
    )]
    pub program_config: Account<'info, ProgramConfig>,

    #[account(
        seeds = [Session::SEED_PREFIX, user_smart_wallet.key().as_ref(), session_key.key().as_ref()],
        constraint = session.session_executor == session_executor.key() @ DappError::UnauthorizedSessionExecutor,
        constraint = session.session_key == session_key.key() @ DappError::UnauthorizedSessionKey,
        constraint = session.status == SessionStatus::Approved @ DappError::SessionNotApproved,
        bump = session.bump
    )]
    pub session: Account<'info, Session>,

    /// We need to pass it as it will be a target program cpi call signer
    #[account(
        seeds = [UserSmartWallet::SEED_PREFIX, user_smart_wallet.smart_wallet_owner.key().as_ref()],
        bump = user_smart_wallet.bump
    )]
    pub user_smart_wallet: Account<'info, UserSmartWallet>,

    #[account(address = session.target_program @ DappError::InvalidTargetProgram)]
    pub target_program: Program<'info>,
}
