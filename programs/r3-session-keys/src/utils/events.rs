use anchor_lang::prelude::*;

#[event]
pub struct SessionCreated {
    pub session: Pubkey,
    pub user_smart_wallet: Pubkey,
    pub session_executor: Pubkey,
    pub session_key: Pubkey,
    pub target_program: Pubkey,
    pub expires_at: i64,
    pub allowed_instructions_discriminators: Vec<u8>,
    pub discriminator_size: u8,
}

#[event]
pub struct SessionUpdated {
    pub session: Pubkey,
    pub user_smart_wallet: Pubkey,
    pub session_executor: Pubkey,
    pub session_key: Pubkey,
    pub target_program: Pubkey,
    pub expires_at: i64,
    pub allowed_instructions_discriminators: Vec<u8>,
    pub discriminator_size: u8,
}

#[event]
pub struct SessionApproved {
    pub session: Pubkey,
    pub user_smart_wallet: Pubkey,
    pub smart_wallet_owner: Pubkey,
    pub session_key: Pubkey,
}

#[event]
pub struct SessionRevoked {
    pub session: Pubkey,
    pub user_smart_wallet: Pubkey,
    pub smart_wallet_owner: Pubkey,
    pub session_key: Pubkey,
}

#[event]
pub struct SessionClosed {
    pub session: Pubkey,
    pub user_smart_wallet: Pubkey,
    pub session_executor: Pubkey,
    pub session_key: Pubkey,
}

#[event]
pub struct ProgramPaused {
    pub program_config: Pubkey,
    pub admin: Pubkey,
}

#[event]
pub struct ProgramUnpaused {
    pub program_config: Pubkey,
    pub admin: Pubkey,
}

#[event]
pub struct SessionExecuted {
    pub session: Pubkey,
    pub user_smart_wallet: Pubkey,
    pub session_executor: Pubkey,
    pub session_key: Pubkey,
    pub target_program: Pubkey,
}
