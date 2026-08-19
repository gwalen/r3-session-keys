use anchor_lang::prelude::*;

use crate::{
    instructions::close_session::CloseSession,
    utils::{common::close_account, events::SessionClosed},
};

pub fn handle(ctx: Context<CloseSession>) -> Result<()> {
    close_account(
        &ctx.accounts.session_executor.to_account_info(),
        &ctx.accounts.session.to_account_info(),
    )?;

    emit!(SessionClosed {
        session: ctx.accounts.session.key(),
        user_smart_wallet: ctx.accounts.user_smart_wallet.key(),
        session_executor: ctx.accounts.session_executor.key(),
        session_key: ctx.accounts.session.session_key,
    });

    Ok(())
}
