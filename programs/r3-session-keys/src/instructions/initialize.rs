use anchor_lang::prelude::*;

use crate::state::program_config::ProgramConfig;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = ProgramConfig::LEN,
        seeds = [ProgramConfig::SEED_PREFIX],
        bump
    )]
    pub program_config: Account<'info, ProgramConfig>,


    pub system_program: Program<'info, System>,
}
