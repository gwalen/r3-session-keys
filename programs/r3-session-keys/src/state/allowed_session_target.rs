use anchor_lang::prelude::*;

// seeds : [b"allowed_target", session_key.to_bytes().as_ref(), target_program_id.to_bytes().as_ref()]
#[account]
#[derive(InitSpace)]
pub struct AllowedSessionTarget {
    // program_id is part of the see we do not need to store it in the struct
    #[max_len(20)]
    pub allowed_instructions_discriminators: Vec<u8>,
    pub bump: u8,
}

impl AllowedSessionTarget {
    pub const LEN: usize = 8 + Self::INIT_SPACE;
   
    // pub const SEED_PREFIX: &'static [u8] = b"allowed_target"; // TODO: why it should be static?
    pub const SEED_PREFIX: &[u8] = b"allowed_target";
}
