use anchor_lang::{
    prelude::Pubkey,
    solana_program::{instruction::Instruction, system_program},
    ToAccountMetas,
};

use super::build_ix;

pub fn counter_pda() -> Pubkey {
    Pubkey::find_program_address(&[mock_program::COUNTER_SEED], &mock_program::ID).0
}

pub fn initialize(payer: Pubkey) -> Instruction {
    build_ix(
        mock_program::ID,
        mock_program::accounts::Initialize {
            payer,
            counter: counter_pda(),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        mock_program::instruction::Initialize {},
    )
}

pub fn increment(authority: Pubkey) -> Instruction {
    build_ix(
        mock_program::ID,
        mock_program::accounts::Increment {
            counter: counter_pda(),
            authority,
        }
        .to_account_metas(None),
        mock_program::instruction::Increment {},
    )
}
