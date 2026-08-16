use anchor_lang::prelude::*;

use crate::{instructions::initialize::Initialize, state::program_config::ProgramConfig};
use crate::state::program_config::ProgramStatus;

pub fn handle(ctx: Context<Initialize>) -> Result<()> {
    let program_config = &mut ctx.accounts.program_config;
    program_config.set_inner(ProgramConfig {
        admin: ctx.accounts.admin.key(),
        status: ProgramStatus::Active,
        bump: ctx.bumps.program_config,
    });
    
    Ok(())
}
