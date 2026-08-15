use anchor_lang::prelude::*;

use crate::{
    instructions::approve_session::ApproveSession,
    state::session::SessionStatus,
};

pub fn handle(
    ctx: Context<ApproveSession>,
) -> Result<()> {
    let session = &mut ctx.accounts.session;

    session.status = SessionStatus::Approved;

    Ok(())
}
