use anchor_lang::prelude::*;

use crate::{
    state::{program_config::ProgramConfig, user_smart_wallet::UserSmartWallet},
    utils::errors::DappError,
};
use crate::state::program_config::ProgramStatus;

#[derive(Accounts)]
#[instruction(user_wallet: Pubkey)]
pub struct CreateSmartWallet<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [ProgramConfig::SEED_PREFIX],
        has_one = admin @ DappError::UnauthorizedAdmin,
        constraint = program_config.status == ProgramStatus::Active @ DappError::ProgramPaused,
        bump = program_config.bump,
    )]
    pub program_config: Account<'info, ProgramConfig>,

    #[account(
        init,
        payer = admin,
        space = 8 + UserSmartWallet::INIT_SPACE,
        seeds = [UserSmartWallet::SEED_PREFIX, user_wallet.key().as_ref()],
        bump
    )]
    pub user_smart_wallet: Account<'info, UserSmartWallet>,

    pub system_program: Program<'info, System>,
}