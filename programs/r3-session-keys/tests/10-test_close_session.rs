mod common;

use {
    common::{
        client, send_tx_expect_error, send_tx_expect_ok, timestamp_from_future, Env,
        DUMMY_ANCHOR_DISCRIMINATOR, TARGET_PROGRAM_PLACEHOLDER,
    },
    solana_keypair::Keypair,
    solana_signer::Signer,
};

#[test]
fn test_close_session_returns_rent_to_executor() {
    let client = client::R3SessionKeysClient::new();
    let mut env = Env::new();
    let initialize_ix = client.initialize(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, initialize_ix, &[&env.admin]);

    let user = Keypair::new();
    let session_executor = Keypair::new();
    let session_key = Keypair::new().pubkey();
    env.svm
        .airdrop(&session_executor.pubkey(), 1_000_000_000)
        .unwrap();

    let (ix, user_smart_wallet, _) = client.create_smart_wallet(env.admin.pubkey(), user.pubkey());
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);

    let (ix, session_pda, _) = client.create_session(
        session_executor.pubkey(),
        user_smart_wallet,
        session_key,
        TARGET_PROGRAM_PLACEHOLDER,
        timestamp_from_future(),
        DUMMY_ANCHOR_DISCRIMINATOR.to_vec(),
        8,
    );
    send_tx_expect_ok(&mut env.svm, ix, &[&session_executor]);

    let executor_balance_before = env.svm.get_balance(&session_executor.pubkey()).unwrap();
    let session_lamports = env.svm.get_account(&session_pda).unwrap().lamports;

    let ix = client.close_session(session_executor.pubkey(), user_smart_wallet, session_key);
    let transaction_fee = send_tx_expect_ok(&mut env.svm, ix, &[&session_executor]);

    assert!(env.svm.get_account(&session_pda).is_none());
    assert_eq!(
        env.svm.get_balance(&session_executor.pubkey()).unwrap(),
        executor_balance_before + session_lamports - transaction_fee
    );
}

#[test]
fn test_close_rejects_non_executor() {
    let client = client::R3SessionKeysClient::new();
    let mut env = Env::new();
    let initialize_ix = client.initialize(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, initialize_ix, &[&env.admin]);

    let user = Keypair::new();
    let session_executor = Keypair::new();
    let other_executor = Keypair::new();
    let session_key = Keypair::new().pubkey();
    env.svm
        .airdrop(&session_executor.pubkey(), 1_000_000_000)
        .unwrap();
    env.svm
        .airdrop(&other_executor.pubkey(), 1_000_000_000)
        .unwrap();

    let (ix, user_smart_wallet, _) = client.create_smart_wallet(env.admin.pubkey(), user.pubkey());
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);

    let (ix, session_pda, _) = client.create_session(
        session_executor.pubkey(),
        user_smart_wallet,
        session_key,
        TARGET_PROGRAM_PLACEHOLDER,
        timestamp_from_future(),
        DUMMY_ANCHOR_DISCRIMINATOR.to_vec(),
        8,
    );
    send_tx_expect_ok(&mut env.svm, ix, &[&session_executor]);

    let ix = client.close_session(other_executor.pubkey(), user_smart_wallet, session_key);
    let error = send_tx_expect_error(&mut env.svm, ix, &[&other_executor]);

    assert!(
        error.contains("Error Code: UnauthorizedSessionExecutor"),
        "{error}"
    );
    assert!(env.svm.get_account(&session_pda).is_some());
}
