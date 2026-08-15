use anchor_lang::prelude::*;

// seeds: [b"session", user_smart_wallet.to_bytes().as_ref(), session_key.to_bytes().as_ref()]
#[account]
#[derive(InitSpace)]
pub struct Session {
    // pub user_smart_wallet: Pubkey, // this will be in the seed
    // who will be allow to use this session, when excutiong the session there will be check if signer is the session owner
    // and another if session owner is using required session key which he earlier reads from blockchain after using creates a session for him 
    // TODO: --NO if user creates a session than use has th ehpemeral session key that must be used for signing
    //         maybe we all session_owner to create a session but only user can approve it or revoke it 
    //         before session is approved is can not be used
    pub session_owner: Pubkey,
    pub session_key: Pubkey,
    pub expires_at: i64,

    // This allocates 4 + 32 * 10 = 324 bytes // 4 bytes for length
    #[max_len(10)]
    pub allowed_writeable_mint_list: Vec<Pubkey>,

    // This allocates 4 + 8 * 10 = 84 bytes // 4 bytes for length
    #[max_len(10)]
    pub mint_limits: Vec<u64>,

    pub status: SessionStatus,
    // pub revoked: bool, // TODO: after revoke, no more operations allowed and than user can close the session and reclaim rent
    pub nonce: u64,
    pub bump: u8,
}

impl Session {
    pub const LEN: usize = 8 + Self::INIT_SPACE;

    pub const SEED_PREFIX: &'static [u8] = b"session";
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum SessionStatus {
    WaitingForApproval,
    Approved,
    Revoked,
}
