use anchor_lang::prelude::*;

use crate::{
    instructions::unpause::Unpause,
    state::program_config::ProgramStatus,
};

pub fn handle(ctx: Context<Unpause>) -> Result<()> {
    ctx.accounts.program_config.status = ProgramStatus::Active;
    msg!("Program unpaused");
    Ok(())
}
