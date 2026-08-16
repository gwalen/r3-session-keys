use anchor_lang::{
    self,
    prelude::Pubkey,
    solana_program::{instruction::Instruction, system_program},
    Discriminator, ToAccountMetas,
};

use super::build_ix;

anchor_lang::declare_program!(mock_program);

pub fn increment_discriminator() -> &'static [u8] {
    mock_program::client::args::Increment::DISCRIMINATOR
}

pub fn counter_pda() -> Pubkey {
    Pubkey::find_program_address(&[mock_program::constants::COUNTER_SEED], &mock_program::ID).0
}

pub fn initialize(payer: Pubkey) -> Instruction {
    build_ix(
        mock_program::ID,
        mock_program::client::accounts::Initialize {
            payer,
            counter: counter_pda(),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        mock_program::client::args::Initialize,
    )
}

pub fn increment(authority: Pubkey) -> Instruction {
    build_ix(
        mock_program::ID,
        mock_program::client::accounts::Increment {
            counter: counter_pda(),
            authority,
        }
        .to_account_metas(None),
        mock_program::client::args::Increment,
    )
}
