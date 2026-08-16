use anchor_lang::prelude::*;

pub mod handlers;
pub mod instructions;
pub mod state;
pub mod utils;

pub use instructions::approve_session::*;
pub use instructions::create_session::*;
pub use instructions::create_smart_wallet::*;
pub use instructions::initialize::*;
pub use instructions::pause::*;
pub use instructions::revoke_session::*;
pub use instructions::unpause::*;
pub use instructions::execute_with_session::*;
pub use utils::errors;
pub use utils::events;

declare_id!("r3xx1495USK8vysHAfL83d9seSofMH77ytxjuhepfFH");

#[program]
pub mod r3_session_keys {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        handlers::initialize::handle(ctx)
    }

    pub fn pause(ctx: Context<Pause>) -> Result<()> {
        handlers::pause::handle(ctx)
    }

    pub fn unpause(ctx: Context<Unpause>) -> Result<()> {
        handlers::unpause::handle(ctx)
    }

    pub fn create_smart_wallet(ctx: Context<CreateSmartWallet>, user_wallet: Pubkey) -> Result<()> {
        handlers::create_smart_wallet::handle(ctx, user_wallet)
    }

    pub fn create_session(
        ctx: Context<CreateSession>,
        session_key: Pubkey,
        target_program: Pubkey,
        expires_at: i64,
        allowed_instructions_discriminators: Vec<u8>,
        discriminator_len: u8,
    ) -> Result<()> {
        handlers::create_session::handle(
            ctx,
            session_key,
            target_program,
            expires_at,
            allowed_instructions_discriminators,
            discriminator_len,
        )
    }

    pub fn approve_session(
        ctx: Context<ApproveSession>,
        _session_key: Pubkey,
    ) -> Result<()> {
        handlers::approve_session::handle(ctx)
    }

    pub fn revoke_session(
        ctx: Context<RevokeSession>,
        _session_key: Pubkey,
    ) -> Result<()> {
        handlers::revoke_session::handle(ctx)
    }

    pub fn execute_with_session(
        ctx: Context<ExecuteWithSession>,
        instruction_data: Vec<u8>,
    ) -> Result<()> {
        handlers::execute_with_session::handle(ctx, instruction_data)
    }

}
