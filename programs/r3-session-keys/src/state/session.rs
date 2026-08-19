use crate::utils::common::read_array_element;
use anchor_lang::prelude::*;

// seeds: [b"session", user_smart_wallet.to_bytes().as_ref(), session_key.to_bytes().as_ref()]
#[account]
#[derive(InitSpace)]
pub struct Session {
    // Bot / external caller allowed to execute this session after the smart-wallet owner approves.
    pub session_executor: Pubkey,
    pub session_key: Pubkey,
    pub target_program: Pubkey,
    pub expires_at: i64,

    // For AnchorV1 that would be 10 discriminators (10 different instructions)
    #[max_len(Session::MAX_DISCRIMINATORS_LEN)]
    pub allowed_instructions_discriminators: Vec<u8>,
    // AnchorV1 has 8 bytes discriminator, Quasar 1 byte, SPL token 1, they can vary
    pub discriminator_size: u8,

    pub status: SessionStatus,
    pub bump: u8,
}

impl Session {
    pub const MAX_DISCRIMINATORS_LEN: usize = 80;

    pub const LEN: usize = 8 + Self::INIT_SPACE;

    pub const SEED_PREFIX: &'static [u8] = b"session";

    pub fn find_pda(user_smart_wallet: &Pubkey, session_key: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                Self::SEED_PREFIX,
                user_smart_wallet.as_ref(),
                session_key.as_ref(),
            ],
            &crate::ID,
        )
    }

    pub fn parse_discriminators(&self) -> Vec<Vec<u8>> {
        let mut discriminators = Vec::new();
        let disc_size = self.discriminator_size as usize;
        let disc_vec_len = self.allowed_instructions_discriminators.len() / disc_size;
        for i in 0..disc_vec_len {
            let disc = read_array_element(&self.allowed_instructions_discriminators, i * disc_size, disc_size);
            discriminators.push(disc.to_vec());
        }
        discriminators
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub enum SessionStatus {
    WaitingForApproval,
    Approved,
    Revoked,
}
