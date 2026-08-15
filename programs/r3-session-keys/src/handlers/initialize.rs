use anchor_lang::prelude::*;

use crate::{instructions::initialize::Initialize, utils::constants::*};

pub fn handle(ctx: Context<Initialize>) -> Result<()> {
    ctx.accounts.counter.count = 0;
    ctx.accounts.counter.authority = ctx.accounts.payer.key();

    let cpi_accounts = anchor_lang::system_program::Transfer {
        from: ctx.accounts.payer.to_account_info(),
        to: ctx.accounts.counter.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(anchor_lang::system_program::ID, cpi_accounts);
    anchor_lang::system_program::transfer(cpi_ctx, HELLO_WORLD_LAMPORTS)?;

    msg!("Hello, world! Counter initialized");
    Ok(())
}
