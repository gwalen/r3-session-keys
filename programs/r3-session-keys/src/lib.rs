use anchor_lang::prelude::*;

pub mod handlers;
pub mod instructions;
pub mod state;
pub mod utils;

pub use instructions::increment::*;
pub use instructions::initialize::*;
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
}
