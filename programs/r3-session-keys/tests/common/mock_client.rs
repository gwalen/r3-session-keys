use anchor_lang::{
    prelude::Pubkey,
    solana_program::{instruction::Instruction, system_program},
    ToAccountMetas,
};

use super::build_ix;

const MOCK_PROGRAM_IDL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/mock-program/idl/mock_program.json"
));

#[derive(serde::Deserialize)]
struct Idl {
    instructions: Vec<IdlInstruction>,
}

#[derive(serde::Deserialize)]
struct IdlInstruction {
    name: String,
    discriminator: Vec<u8>,
}

pub fn instruction_discriminator(instruction_name: &str) -> Vec<u8> {
    let idl: Idl = serde_json::from_str(MOCK_PROGRAM_IDL).expect("mock program IDL must be valid");
    idl.instructions
        .into_iter()
        .find(|instruction| instruction.name == instruction_name)
        .unwrap_or_else(|| panic!("instruction `{instruction_name}` not found in mock program IDL"))
        .discriminator
}

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
