use anchor_lang::prelude::*;

use crate::{
    instructions::update_session::UpdateSession,
    state::session::SessionStatus,
    utils::{common::validate_session_params, events::SessionUpdated},
};

pub fn handle(
    ctx: Context<UpdateSession>,
    target_program: Pubkey,
    expires_at: i64,
    allowed_instructions_discriminators: Vec<u8>,
    discriminator_len: u8,
) -> Result<()> {
    validate_session_params(
        &allowed_instructions_discriminators,
        discriminator_len,
        expires_at,
    )?;

    let session = &mut ctx.accounts.session;
    session.target_program = target_program;
    session.expires_at = expires_at;
    session.allowed_instructions_discriminators = allowed_instructions_discriminators;
    session.discriminator_size = discriminator_len;
    session.status = SessionStatus::WaitingForApproval;

    emit!(SessionUpdated {
        session: session.key(),
        user_smart_wallet: ctx.accounts.user_smart_wallet.key(),
        session_executor: ctx.accounts.session_executor.key(),
        session_key: session.session_key,
        target_program,
        expires_at,
        allowed_instructions_discriminators: session.allowed_instructions_discriminators.clone(),
        discriminator_size: discriminator_len,
    });

    Ok(())
}
