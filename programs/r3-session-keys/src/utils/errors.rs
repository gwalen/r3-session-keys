use anchor_lang::prelude::*;

#[error_code]
pub enum DappError {
    #[msg("Only the counter authority can update this counter")]
    Unauthorized,
    #[msg("Counter has reached the maximum value")]
    CounterOverflow,
    #[msg("Only the program admin can pause or unpause the program")]
    UnauthorizedAdmin,
    #[msg("Program is already paused")]
    AlreadyPaused,
    #[msg("Program is already active")]
    AlreadyActive,
    #[msg("Program paused")]
    ProgramPaused,
    #[msg("User smart wallet not found")]
    UserSmartWalletNotFound,
}
