use anchor_lang::prelude::*;

use crate::{
    instructions::revoke_session::RevokeSession,
    state::session::SessionStatus,
};

pub fn handle(
    ctx: Context<RevokeSession>,
) -> Result<()> {
    let session = &mut ctx.accounts.session;

    session.status = SessionStatus::Revoked;

    Ok(())
}
