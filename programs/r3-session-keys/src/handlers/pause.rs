use anchor_lang::prelude::*;

use crate::{
    instructions::pause::Pause,
    state::program_config::ProgramStatus,
    utils::events::ProgramPaused,
};

pub fn handle(ctx: Context<Pause>) -> Result<()> {
    ctx.accounts.program_config.status = ProgramStatus::Paused;
    emit!(ProgramPaused {
        program_config: ctx.accounts.program_config.key(),
        admin: ctx.accounts.admin.key(),
    });
    Ok(())
}
