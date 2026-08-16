use anchor_lang::prelude::*;

use crate::{
    instructions::revoke_session::RevokeSession,
    state::session::SessionStatus,
    utils::events::SessionRevoked,
};

pub fn handle(
    ctx: Context<RevokeSession>,
) -> Result<()> {
    let session = &mut ctx.accounts.session;

    session.status = SessionStatus::Revoked;

    emit!(SessionRevoked {
        session: session.key(),
        user_smart_wallet: ctx.accounts.user_smart_wallet.key(),
        smart_wallet_owner: ctx.accounts.smart_wallet_owner.key(),
        session_key: session.session_key,
    });

    Ok(())
}
