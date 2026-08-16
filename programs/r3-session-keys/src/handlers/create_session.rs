use anchor_lang::prelude::*;

use crate::{
    instructions::create_session::CreateSession,
    state::session::{Session, SessionStatus},
};

pub fn handle(
    ctx: Context<CreateSession>,
    session_key: Pubkey,
    expires_at: i64,
    allowed_instructions_discriminators: Vec<u8>,
    discriminator_len: u8,
) -> Result<()> {
    ctx.accounts.session.set_inner(Session {
        session_owner: ctx.accounts.session_owner.key(),
        session_key,
        expires_at,
        // TODO: implement later if enough time, or just one mint
        // allowed_writeable_mint_list: vec![],
        // mint_limits: vec![],
        allowed_instructions_discriminators,
        discriminator_len,
        status: SessionStatus::WaitingForApproval,
        nonce: 0,
        bump: ctx.bumps.session,
    });

    Ok(())
}
