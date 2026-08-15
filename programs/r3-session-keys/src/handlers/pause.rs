use anchor_lang::prelude::*;

use crate::{
    instructions::pause::Pause,
    state::program_config::ProgramStatus,
};

pub fn handle(ctx: Context<Pause>) -> Result<()> {
    ctx.accounts.program_config.status = ProgramStatus::Paused;
    msg!("Program paused");
    Ok(())
}
