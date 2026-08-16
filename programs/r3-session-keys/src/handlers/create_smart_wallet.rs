use anchor_lang::prelude::*;

use crate::{
    instructions::create_smart_wallet::CreateSmartWallet,
    state::user_smart_wallet::UserSmartWallet,
};

pub fn handle(ctx: Context<CreateSmartWallet>, user_wallet: Pubkey) -> Result<()> {
    let user_smart_wallet = &mut ctx.accounts.user_smart_wallet;

    user_smart_wallet.set_inner(UserSmartWallet {
        smart_wallet_owner: user_wallet,
        bump: ctx.bumps.user_smart_wallet,
    });

    Ok(())
}