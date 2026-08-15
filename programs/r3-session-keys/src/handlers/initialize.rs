use anchor_lang::prelude::*;

use crate::{instructions::initialize::Initialize, state::program_config::ProgramConfig, utils::constants::*};
use crate::state::program_config::ProgramStatus;

pub fn handle(ctx: Context<Initialize>) -> Result<()> {
    // ----- TODO: later remove it 
    ctx.accounts.counter.count = 0;
    ctx.accounts.counter.authority = ctx.accounts.admin.key();

    let cpi_accounts = anchor_lang::system_program::Transfer {
        from: ctx.accounts.admin.to_account_info(),
        to: ctx.accounts.counter.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(anchor_lang::system_program::ID, cpi_accounts);
    anchor_lang::system_program::transfer(cpi_ctx, HELLO_WORLD_LAMPORTS)?;

    msg!("Hello, world! Counter initialized");
    // ----------------

    let program_config = &mut ctx.accounts.program_config;
    program_config.set_inner(ProgramConfig {
        admin: ctx.accounts.admin.key(),
        status: ProgramStatus::Active,
        bump: ctx.bumps.program_config,
    });
    
    Ok(())
}
