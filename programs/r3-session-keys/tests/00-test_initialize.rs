mod common;

use {
    anchor_lang::{prelude::Pubkey, AccountDeserialize},
    common::client::R3SessionKeysClient,
    litesvm::LiteSVM,
    r3_session_keys::{
        state::{
            counter::Counter,
            program_config::{ProgramConfig, ProgramStatus},
        },
        utils::constants::COUNTER_SEED,
    },
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

fn send_and_log(svm: &mut LiteSVM, tx: VersionedTransaction) {
    match svm.send_transaction(tx) {
        Ok(meta) => println!("{}", meta.pretty_logs()),
        Err(e) => {
            println!("Transaction failed: {:?}", e.err);
            println!("{}", e.meta.pretty_logs());
            panic!("send_transaction failed: {:?}", e.err);
        }
    }
}

#[test]
fn test_initialize() {
    let program_id = r3_session_keys::id();
    let payer = Keypair::new();
    let counter = Pubkey::find_program_address(&[COUNTER_SEED], &program_id).0;
    let (program_config, program_config_bump) =
        Pubkey::find_program_address(&[ProgramConfig::SEED_PREFIX], &program_id);
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/r3_session_keys.so"
    ));
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();
    let client = R3SessionKeysClient::new();

    println!("XXX program id: {:?}", program_id);

    let ix = client.initialize(payer.pubkey());

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();

    send_and_log(&mut svm, tx);

    let counter_account = svm.get_account(&counter).unwrap();
    let mut data: &[u8] = &counter_account.data;
    let counter_state = Counter::try_deserialize(&mut data).unwrap();
    assert_eq!(counter_state.count, 0);
    assert_eq!(counter_state.authority, payer.pubkey());

    let program_config_account = svm.get_account(&program_config).unwrap();
    let mut data: &[u8] = &program_config_account.data;
    let program_config_state = ProgramConfig::try_deserialize(&mut data).unwrap();
    assert_eq!(program_config_state.admin, payer.pubkey());
    assert_eq!(program_config_state.status, ProgramStatus::Active);
    assert_eq!(program_config_state.bump, program_config_bump);

    let ix = client.increment(payer.pubkey());

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();

    send_and_log(&mut svm, tx);

    let counter_account = svm.get_account(&counter).unwrap();
    let mut data: &[u8] = &counter_account.data;
    let counter_state = Counter::try_deserialize(&mut data).unwrap();
    assert_eq!(counter_state.count, 1);
    assert_eq!(counter_state.authority, payer.pubkey());
}
