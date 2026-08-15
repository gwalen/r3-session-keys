use anchor_lang::prelude::*;

use crate::utils::constants::COUNTER_SEED;

#[account]
#[derive(InitSpace)]
pub struct Counter {
    pub count: u64,
    pub authority: Pubkey,
}

impl Counter {
    pub fn find_pda() -> (Pubkey, u8) {
        Pubkey::find_program_address(&[COUNTER_SEED], &crate::ID)
    }
}
