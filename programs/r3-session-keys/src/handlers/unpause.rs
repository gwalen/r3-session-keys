use anchor_lang::prelude::*;

use crate::{
    instructions::unpause::Unpause,
    state::program_config::ProgramStatus,
    utils::events::ProgramUnpaused,
};

pub fn handle(ctx: Context<Unpause>) -> Result<()> {
    ctx.accounts.program_config.status = ProgramStatus::Active;
    emit!(ProgramUnpaused {
        program_config: ctx.accounts.program_config.key(),
        admin: ctx.accounts.admin.key(),
    });
    Ok(())
}
