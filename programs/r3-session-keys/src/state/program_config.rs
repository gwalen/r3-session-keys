use anchor_lang::prelude::*;

// seeds : [b"program_config"]
#[account]
#[derive(InitSpace)]
pub struct ProgramConfig {
    pub admin: Pubkey,
    pub status: ProgramStatus,
    pub bump: u8,
}

impl ProgramConfig {
    pub const LEN: usize = 8 + Self::INIT_SPACE;

    pub const SEED_PREFIX: &'static [u8] = b"program_config";
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub enum ProgramStatus {
    Active,
    Paused,
}
