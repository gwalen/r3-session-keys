use anchor_lang::prelude::*;

use crate::{
    instructions::approve_session::ApproveSession,
    state::session::SessionStatus,
    utils::events::SessionApproved,
};

pub fn handle(
    ctx: Context<ApproveSession>,
) -> Result<()> {
    let session = &mut ctx.accounts.session;

    session.status = SessionStatus::Approved;

    emit!(SessionApproved {
        session: session.key(),
        user_smart_wallet: ctx.accounts.user_smart_wallet.key(),
        smart_wallet_owner: ctx.accounts.smart_wallet_owner.key(),
        session_key: session.session_key,
    });

    Ok(())
}
