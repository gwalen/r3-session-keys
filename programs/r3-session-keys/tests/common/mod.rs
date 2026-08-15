#![allow(dead_code)]

pub mod client;
mod env;

pub use env::Env;

use {
    anchor_lang::{prelude::Pubkey, solana_program::instruction::Instruction, AccountDeserialize},
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

pub fn send(env: &mut Env, ix: Instruction) {
    send_tx(&mut env.svm, ix, &[&env.admin]);
}

pub fn send_err(env: &mut Env, ix: Instruction) -> String {
    send_tx_err(&mut env.svm, ix, &[&env.admin])
}

pub fn load<T: AccountDeserialize>(env: &Env, key: &Pubkey) -> T {
    let account = env.svm.get_account(key).unwrap();
    let mut data: &[u8] = &account.data;
    T::try_deserialize(&mut data).unwrap()
}

pub fn send_tx(svm: &mut LiteSVM, ix: Instruction, signers: &[&Keypair]) {
    let payer = signers[0];
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    match svm.send_transaction(tx) {
        Ok(meta) => println!("{}", meta.pretty_logs()),
        Err(e) => {
            println!("Transaction failed: {:?}", e.err);
            println!("{}", e.meta.pretty_logs());
            panic!("send_transaction failed: {:?}", e.err);
        }
    }
}

fn send_tx_err(svm: &mut LiteSVM, ix: Instruction, signers: &[&Keypair]) -> String {
    let payer = signers[0];
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash); // TODO think twice
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    match svm.send_transaction(tx) {
        Ok(meta) => {
            println!("{}", meta.pretty_logs());
            panic!("expected transaction to fail");
        }
        Err(e) => {
            println!("Transaction failed as expected: {:?}", e.err);
            println!("{}", e.meta.pretty_logs());
            svm.expire_blockhash();
            format!("{:#?} {}", e.err, e.meta.pretty_logs())
        }
    }
}

pub fn assert_program_paused(logs: &str) {
    assert!(
        logs.contains("ProgramPaused") || logs.contains("Program paused"),
        "expected ProgramPaused, got: {logs}"
    );
}
