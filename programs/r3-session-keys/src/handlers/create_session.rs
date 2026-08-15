use anchor_lang::prelude::*;

use crate::{
    instructions::create_session::CreateSession,
    state::session::{Session, SessionStatus},
};

pub fn handle(
    ctx: Context<CreateSession>,
    session_key: Pubkey,
    _smart_wallet: Pubkey,
    // TODO: add expiration time as parameter in timestamp
) -> Result<()> {
    ctx.accounts.session.set_inner(Session {
        session_owner: ctx.accounts.session_owner.key(),
        session_key,
        expires_at: 0,
        allowed_writeable_mint_list: vec![],
        mint_limits: vec![], // TODO: remove for now, implement later if enough time
        status: SessionStatus::WaitingForApproval,
        nonce: 0,
        bump: ctx.bumps.session,
    });

    Ok(())
}
