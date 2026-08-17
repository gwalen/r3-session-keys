use anchor_lang::prelude::*;

use crate::{
    instructions::create_session::CreateSession,
    state::session::{Session, SessionStatus},
    utils::errors::DappError,
    utils::events::SessionCreated,
};

pub fn handle(
    ctx: Context<CreateSession>,
    session_key: Pubkey,
    target_program: Pubkey,
    expires_at: i64,
    allowed_instructions_discriminators: Vec<u8>,
    discriminator_len: u8,
) -> Result<()> {
    // discriminator_len = 0 would divide by zero in Session::parse_discriminators
    require!(discriminator_len > 0, DappError::InvalidDiscriminatorSize);
    require!(
        !allowed_instructions_discriminators.is_empty()
            && allowed_instructions_discriminators.len() % discriminator_len as usize == 0,
        DappError::InvalidDiscriminatorListLength
    );
    require!(
        expires_at > Clock::get()?.unix_timestamp,
        DappError::SessionExpirationInPast
    );

    ctx.accounts.session.set_inner(Session {
        session_executor: ctx.accounts.session_executor.key(),
        session_key,
        target_program,
        expires_at,
        allowed_instructions_discriminators,
        discriminator_size: discriminator_len,
        status: SessionStatus::WaitingForApproval,
        bump: ctx.bumps.session,
    });

    emit!(SessionCreated {
        session: ctx.accounts.session.key(),
        user_smart_wallet: ctx.accounts.user_smart_wallet.key(),
        session_executor: ctx.accounts.session_executor.key(),
        session_key,
        target_program,
        expires_at,
        allowed_instructions_discriminators: ctx
            .accounts
            .session
            .allowed_instructions_discriminators
            .clone(),
        discriminator_size: discriminator_len,
    });

    Ok(())
}
