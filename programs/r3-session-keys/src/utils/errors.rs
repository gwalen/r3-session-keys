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
    #[msg("Unauthorized session executor")]
    UnauthorizedSessionExecutor,
    #[msg("Unauthorized smart wallet owner")]
    UnauthorizedSmartWalletOwner,
    #[msg("Unauthorized session key")]
    UnauthorizedSessionKey,
    #[msg("Target program does not match the session target program")]
    InvalidTargetProgram,
    #[msg("Session not approved")]
    SessionNotApproved,
    #[msg("Session expired")]
    SessionExpired,
    #[msg("Empty instruction data")]
    EmptyInstructionData,
    #[msg("Not allowed to call smart wallet program")]
    NotAllowedToCallSmartWalletProgram,
    #[msg("Remaining accounts contains an unexpected program-owned account")]
    RemainingAccountsContainsProgramOwnedAccount,
    #[msg("Remaining accounts contains session key")]
    RemainingAccountsContainsSessionKey,
    #[msg("Multiple user smart wallet accounts found")]
    MultipleUserSmartWalletAccounts,
    #[msg("User smart wallet account is writable")]
    UserSmartWalletAccountIsWritable,
    #[msg("Not allowed instruction discriminator")]
    NotAllowedInstructionDiscriminator,
}
