use anchor_lang::prelude::*;

use crate::{
    state::session::Session,
    utils::errors::DappError,
    state::{program_config::ProgramConfig, user_smart_wallet::UserSmartWallet},
};


#[derive(Accounts)]
#[instruction(_session_key: Pubkey)]
pub struct RevokeSession<'info> {
    pub smart_wallet_owner: Signer<'info>,

    // action and must stay available while the program is paused (if there is an incident),
    // so owners can kill compromised sessions before the program is unpaused.
    #[account(
        seeds = [ProgramConfig::SEED_PREFIX],
        bump = program_config.bump,
    )]
    pub program_config: Account<'info, ProgramConfig>,

    // deserialize will check if accounts exists (program ownership and structure)
    #[account(
        seeds = [UserSmartWallet::SEED_PREFIX, smart_wallet_owner.key().as_ref()],
        constraint = user_smart_wallet.smart_wallet_owner == smart_wallet_owner.key() @ DappError::UnauthorizedSmartWalletOwner,
        bump = user_smart_wallet.bump,
    )]
    pub user_smart_wallet: Account<'info, UserSmartWallet>,

    #[account(
        mut,
        seeds = [Session::SEED_PREFIX, user_smart_wallet.key().as_ref(), _session_key.key().as_ref()],
        bump = session.bump
    )]
    pub session: Box<Account<'info, Session>>,
}
