use anchor_lang::prelude::*;

// seeds : [b"user_smart_wallet", owner.to_bytes().as_ref()]
#[account]
#[derive(InitSpace)]
pub struct UserSmartWallet {
    pub owner: Pubkey,
    pub bump: u8,
}

impl UserSmartWallet {
    pub const LEN: usize = 8 + Self::INIT_SPACE;

    pub const SEED_PREFIX: &'static [u8] = b"user_smart_wallet";
}
