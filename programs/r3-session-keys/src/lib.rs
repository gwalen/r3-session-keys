use anchor_lang::prelude::*;

pub mod handlers;
pub mod instructions;
pub mod state;
pub mod utils;

pub use instructions::approve_session::*;
pub use instructions::create_session::*;
pub use instructions::create_smart_wallet::*;
pub use instructions::increment::*;
pub use instructions::initialize::*;
pub use instructions::pause::*;
pub use instructions::unpause::*;
pub use utils::constants;
pub use utils::errors;

declare_id!("r3xx1495USK8vysHAfL83d9seSofMH77ytxjuhepfFH");

#[program]
pub mod r3_session_keys {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        handlers::initialize::handle(ctx)
    }

    pub fn increment(ctx: Context<Increment>) -> Result<()> {
        handlers::increment::handle(ctx)
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
        _smart_wallet: Pubkey, // TODO: do not ass as param but as account in ix
        expires_at: i64,
    ) -> Result<()> {
        handlers::create_session::handle(ctx, session_key, expires_at)
    }

    pub fn approve_session(
        ctx: Context<ApproveSession>,
        _session_key: Pubkey,
        _smart_wallet: Pubkey, // TODO: do not ass as param but as account in ix
    ) -> Result<()> {
        handlers::approve_session::handle(ctx)
    }
}
