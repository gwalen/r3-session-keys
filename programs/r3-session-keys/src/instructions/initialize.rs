use anchor_lang::prelude::*;

use crate::{state::{counter::Counter, program_config::ProgramConfig}, utils::constants::*};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    // TODO: later remove it
    #[account(
        init,
        payer = admin,
        space = 8 + Counter::INIT_SPACE,
        seeds = [COUNTER_SEED],
        bump
    )]
    pub counter: Account<'info, Counter>,

    #[account(
        init,
        payer = admin,
        space = 8 + ProgramConfig::INIT_SPACE,
        seeds = [ProgramConfig::SEED_PREFIX.as_ref()],
        bump
    )]
    pub program_config: Account<'info, ProgramConfig>,


    pub system_program: Program<'info, System>,
}