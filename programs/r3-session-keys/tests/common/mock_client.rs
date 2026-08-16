use anchor_client::{Client, Cluster, Program};
use anchor_lang::{
    self,
    prelude::Pubkey,
    solana_program::{instruction::Instruction, system_program},
    Discriminator,
};
use anchor_spl::{associated_token, token, token_2022};
use solana_keypair::Keypair;
use std::rc::Rc;

anchor_lang::declare_program!(mock_program);

use mock_program::client::{accounts::*, args};

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

    pub fn initialize_pool(&self, payer: Pubkey, deposit_mint: Pubkey) -> Instruction {
        let pool = pool_pda(&deposit_mint);
        let lp_mint = lp_mint_pda(&pool);
        self.program
            .request()
            .accounts(InitializePool {
                payer,
                pool,
                deposit_mint,
                lp_mint,
                vault: vault_address(&pool, &deposit_mint),
                deposit_token_program: token::ID,
                token_2022_program: token_2022::ID,
                associated_token_program: associated_token::ID,
                system_program: system_program::ID,
            })
            .args(args::InitializePool)
            .instructions()
            .remove(0)
    }

    pub fn deposit(&self, user: Pubkey, deposit_mint: Pubkey, amount: u64) -> Instruction {
        let pool = pool_pda(&deposit_mint);
        let lp_mint = lp_mint_pda(&pool);
        self.program
            .request()
            .accounts(Deposit {
                user,
                pool,
                deposit_mint,
                user_deposit_account: user_deposit_account(&user, &deposit_mint),
                vault: vault_address(&pool, &deposit_mint),
                lp_mint,
                user_lp_account: user_lp_account(&user, &lp_mint),
                deposit_token_program: token::ID,
                token_2022_program: token_2022::ID,
            })
            .args(args::Deposit { amount })
            .instructions()
            .remove(0)
    }

    pub fn withdraw(&self, user: Pubkey, deposit_mint: Pubkey, amount: u64) -> Instruction {
        let pool = pool_pda(&deposit_mint);
        let lp_mint = lp_mint_pda(&pool);
        self.program
            .request()
            .accounts(Withdraw {
                user,
                pool,
                deposit_mint,
                user_deposit_account: user_deposit_account(&user, &deposit_mint),
                vault: vault_address(&pool, &deposit_mint),
                lp_mint,
                user_lp_account: user_lp_account(&user, &lp_mint),
                deposit_token_program: token::ID,
                token_2022_program: token_2022::ID,
            })
            .args(args::Withdraw { amount })
            .instructions()
            .remove(0)
    }
}

pub fn increment_discriminator() -> &'static [u8] {
    args::Increment::DISCRIMINATOR
}

pub fn deposit_discriminator() -> &'static [u8] {
    args::Deposit::DISCRIMINATOR
}

pub fn withdraw_discriminator() -> &'static [u8] {
    args::Withdraw::DISCRIMINATOR
}

pub fn counter_pda() -> Pubkey {
    Pubkey::find_program_address(&[mock_program::constants::COUNTER_SEED], &mock_program::ID).0
}

pub fn pool_pda(deposit_mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[mock_program::constants::POOL_SEED, deposit_mint.as_ref()],
        &mock_program::ID,
    )
    .0
}

pub fn lp_mint_pda(pool: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[mock_program::constants::LP_MINT_SEED, pool.as_ref()],
        &mock_program::ID,
    )
    .0
}

pub fn vault_address(pool: &Pubkey, deposit_mint: &Pubkey) -> Pubkey {
    associated_token::get_associated_token_address_with_program_id(pool, deposit_mint, &token::ID)
}

pub fn user_deposit_account(user: &Pubkey, deposit_mint: &Pubkey) -> Pubkey {
    associated_token::get_associated_token_address_with_program_id(user, deposit_mint, &token::ID)
}

pub fn user_lp_account(user: &Pubkey, lp_mint: &Pubkey) -> Pubkey {
    associated_token::get_associated_token_address_with_program_id(user, lp_mint, &token_2022::ID)
}
