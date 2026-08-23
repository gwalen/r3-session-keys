use anchor_lang::prelude::*;

use crate::{
    state::program_config::{ProgramConfig, ProgramStatus},
    utils::errors::DappError,
};

#[derive(Accounts)]
pub struct Unpause<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [ProgramConfig::SEED_PREFIX],
        has_one = admin @ DappError::UnauthorizedAdmin,
        constraint = program_config.status == ProgramStatus::Paused @ DappError::AlreadyActive,
        bump = program_config.bump,
    )]
    pub program_config: Account<'info, ProgramConfig>,
}
