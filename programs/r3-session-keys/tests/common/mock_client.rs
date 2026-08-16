use anchor_client::{Client, Cluster, Program};
use anchor_lang::{
    self,
    prelude::Pubkey,
    solana_program::{instruction::Instruction, system_program},
    Discriminator,
};
use solana_keypair::Keypair;
use std::rc::Rc;

anchor_lang::declare_program!(mock_program);

pub struct MockClient {
    program: Program<Rc<Keypair>>,
}

impl MockClient {
    pub fn new() -> Self {
        let client = Client::new(Cluster::Localnet, Rc::new(Keypair::new()));
        let program = client.program(mock_program::ID).unwrap();
        Self { program }
    }

    pub fn initialize(&self, payer: Pubkey) -> Instruction {
        self.program
            .request()
            .accounts(mock_program::client::accounts::Initialize {
                payer,
                counter: counter_pda(),
                system_program: system_program::ID,
            })
            .args(mock_program::client::args::Initialize)
            .instructions()
            .remove(0)
    }

    pub fn increment(&self, authority: Pubkey) -> Instruction {
        self.program
            .request()
            .accounts(mock_program::client::accounts::Increment {
                counter: counter_pda(),
                authority,
            })
            .args(mock_program::client::args::Increment)
            .instructions()
            .remove(0)
    }
}

pub fn increment_discriminator() -> &'static [u8] {
    mock_program::client::args::Increment::DISCRIMINATOR
}

pub fn counter_pda() -> Pubkey {
    Pubkey::find_program_address(&[mock_program::constants::COUNTER_SEED], &mock_program::ID).0
}
