use anchor_lang::prelude::*;

use crate::{
    state::session::Session,
    utils::errors::DappError,
    state::{program_config::ProgramConfig, user_smart_wallet::UserSmartWallet},
    state::program_config::ProgramStatus,
};


#[derive(Accounts)]
#[instruction(session_key: Pubkey)]
pub struct CreateSession<'info> {
    #[account(mut)]
    pub session_executor: Signer<'info>,

    #[account(
        seeds = [ProgramConfig::SEED_PREFIX],
        constraint = program_config.status == ProgramStatus::Active @ DappError::ProgramPaused,
        bump = program_config.bump,
    )]
    pub program_config: Account<'info, ProgramConfig>,

    // deserialize will check if accounts exists (program ownership and structure)
    pub user_smart_wallet: Account<'info, UserSmartWallet>,

    #[account(
        init,
        payer = session_executor,
        space = 8 + Session::INIT_SPACE,
        seeds = [Session::SEED_PREFIX, user_smart_wallet.key().as_ref(), session_key.key().as_ref()],
        bump
    )]
    pub session: Box<Account<'info, Session>>,

    pub system_program: Program<'info, System>,
}
