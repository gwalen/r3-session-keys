use anchor_lang::prelude::*;

use crate::{
    state::program_config::{ProgramConfig, ProgramStatus},
    utils::errors::DappError,
};

#[derive(Accounts)]
pub struct Pause<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [ProgramConfig::SEED_PREFIX],
        bump = program_config.bump,
        has_one = admin @ DappError::UnauthorizedAdmin,
        constraint = program_config.status == ProgramStatus::Active @ DappError::AlreadyPaused,
    )]
    pub program_config: Account<'info, ProgramConfig>,
}
