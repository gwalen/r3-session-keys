use anchor_lang::prelude::*;

use crate::{
    instructions::increment::Increment,
    utils::{constants::*, errors::DappError},
};

pub fn handle(ctx: Context<Increment>) -> Result<()> {
    require_keys_eq!(
        ctx.accounts.counter.authority,
        ctx.accounts.authority.key(),
        DappError::Unauthorized,
    );
    require!(
        ctx.accounts.counter.count < MAX_COUNT,
        DappError::CounterOverflow,
    );

    ctx.accounts.counter.count += 1;
    msg!("Hello, world! Counter is now {}", ctx.accounts.counter.count);
    Ok(())
}
