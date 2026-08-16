#![allow(dead_code)]

pub mod client;
mod env;
pub mod mock_client;

pub use env::Env;

use {
    anchor_lang::{prelude::Pubkey, solana_program::instruction::Instruction, AccountDeserialize},
    litesvm::LiteSVM,
    r3_session_keys::state::program_config::{ProgramConfig, ProgramStatus},
    solana_keypair::Keypair,
    solana_message::{v0, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

fn signed_v0_tx(svm: &LiteSVM, ix: Instruction, signers: &[&Keypair]) -> VersionedTransaction {
    let payer = signers[0];
    let msg = v0::Message::try_compile(
        &payer.pubkey(),
        &[ix],
        &[], // LUT
        svm.latest_blockhash(),
    )
    .unwrap();
    VersionedTransaction::try_new(VersionedMessage::V0(msg), signers).unwrap()
}

pub fn load<T: AccountDeserialize>(env: &Env, key: &Pubkey) -> T {
    let account = env.svm.get_account(key).unwrap();
    let mut data: &[u8] = &account.data;
    T::try_deserialize(&mut data).unwrap()
}

pub fn send_tx_expect_ok(svm: &mut LiteSVM, ix: Instruction, signers: &[&Keypair]) {
    let tx = signed_v0_tx(svm, ix, signers);
    let result = svm.send_transaction(tx);
    advance_blockhash(svm);
    match result {
        Ok(meta) => println!("{}", meta.pretty_logs()),
        Err(e) => {
            println!("Transaction failed: {:?}", e.err);
            println!("{}", e.meta.pretty_logs());
            panic!("send_transaction failed: {:?}", e.err);
        }
    }
}

pub fn send_tx_expect_error(svm: &mut LiteSVM, ix: Instruction, signers: &[&Keypair]) -> String {
    let tx = signed_v0_tx(svm, ix, signers);
    let result = svm.send_transaction(tx);
    advance_blockhash(svm);
    match result {
        Ok(meta) => {
            println!("{}", meta.pretty_logs());
            panic!("expected transaction to fail");
        }
        Err(e) => {
            println!("Transaction failed as expected: {:?}", e.err);
            println!("{}", e.meta.pretty_logs());
            format!("{:#?} {}", e.err, e.meta.pretty_logs())
        }
    }
}

// LiteSVM does not produce blocks. latest_blockhash() stays the same until something calls expire_blockhash()
// if would retry a transaction it would have the same blockhash and there for the same signature and it would fail
// so we simulate a blockhash advance by calling expire_blockhash()
pub fn advance_blockhash(svm: &mut LiteSVM) {
    svm.expire_blockhash();
}

pub fn assert_program_paused(env: &Env) {
    let config: ProgramConfig = load(env, &env.program_config);
    assert_eq!(config.status, ProgramStatus::Paused);
}
