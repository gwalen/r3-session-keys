mod common;

use {
    anchor_lang::AccountSerialize,
    common::{
        client, load, mock_client, send_tx_expect_error, send_tx_expect_ok, timestamp_from_future,
        Env, DUMMY_ANCHOR_DISCRIMINATOR, TARGET_PROGRAM_PLACEHOLDER,
    },
    mock_client::mock_program::accounts::Counter,
    r3_session_keys::state::session::{Session, SessionStatus},
    solana_keypair::Keypair,
    solana_signer::Signer,
};

fn load_mock_program(env: &mut Env) {
    let bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/mock-program/mock_program.so"
    ));
    env.svm
        .add_program(mock_client::mock_program::ID, bytes)
        .unwrap();
}

fn initialize_mock_counter(
    env: &mut Env,
    mock_client: &mock_client::MockClient,
    authority: anchor_lang::prelude::Pubkey,
) {
    let counter = mock_client::counter_pda();
    let initialize_ix = mock_client.initialize(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, initialize_ix, &[&env.admin]);

    let mut counter_account = env.svm.get_account(&counter).unwrap();
    let counter_state = Counter {
        count: 0,
        authority,
    };
    let mut account_data = counter_account.data.as_mut_slice();
    counter_state.try_serialize(&mut account_data).unwrap();
    env.svm.set_account(counter, counter_account).unwrap();
}

#[test]
fn test_update_session_resets_approval() {
    let client = client::R3SessionKeysClient::new();
    let mut env = Env::new();
    let initialize_ix = client.initialize(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, initialize_ix, &[&env.admin]);

    load_mock_program(&mut env);
    let mock_client = mock_client::MockClient::new();
    let smart_wallet_owner = Keypair::new();
    let session_key = Keypair::new();
    let (create_wallet, user_smart_wallet, _) =
        client.create_smart_wallet(env.admin.pubkey(), smart_wallet_owner.pubkey());
    send_tx_expect_ok(&mut env.svm, create_wallet, &[&env.admin]);
    initialize_mock_counter(&mut env, &mock_client, user_smart_wallet);

    let expires_at_t1 = timestamp_from_future();
    let expires_at_t2 = expires_at_t1 + 1000;
    let discriminator_set_a = DUMMY_ANCHOR_DISCRIMINATOR.to_vec();
    let discriminator_set_b = mock_client::increment_discriminator().to_vec();
    let (create_session, session, _) = client.create_session(
        env.admin.pubkey(),
        user_smart_wallet,
        session_key.pubkey(),
        TARGET_PROGRAM_PLACEHOLDER,
        expires_at_t1,
        discriminator_set_a,
        8,
    );
    send_tx_expect_ok(&mut env.svm, create_session, &[&env.admin]);

    let approve_session = client.approve_session(
        smart_wallet_owner.pubkey(),
        user_smart_wallet,
        session_key.pubkey(),
    );
    send_tx_expect_ok(
        &mut env.svm,
        approve_session,
        &[&env.admin, &smart_wallet_owner],
    );

    let update_session = client.update_session(
        env.admin.pubkey(),
        user_smart_wallet,
        session_key.pubkey(),
        mock_client::mock_program::ID,
        expires_at_t2,
        discriminator_set_b.clone(),
        u8::try_from(discriminator_set_b.len()).unwrap(),
    );
    send_tx_expect_ok(&mut env.svm, update_session, &[&env.admin]);

    let updated: Session = load(&env, &session);
    assert_eq!(updated.session_executor, env.admin.pubkey());
    assert_eq!(updated.session_key, session_key.pubkey());
    assert_eq!(updated.target_program, mock_client::mock_program::ID);
    assert_eq!(updated.expires_at, expires_at_t2);
    assert_eq!(
        updated.allowed_instructions_discriminators,
        discriminator_set_b
    );
    assert_eq!(updated.discriminator_size, 8);
    assert_eq!(updated.status, SessionStatus::WaitingForApproval);

    let increment_ix = mock_client.increment(user_smart_wallet);
    let execute = client.execute_with_session(
        env.admin.pubkey(),
        session_key.pubkey(),
        user_smart_wallet,
        increment_ix.clone(),
    );
    let error = send_tx_expect_error(&mut env.svm, execute, &[&env.admin, &session_key]);
    assert!(error.contains("Error Code: SessionNotApproved"), "{error}");

    let approve_session = client.approve_session(
        smart_wallet_owner.pubkey(),
        user_smart_wallet,
        session_key.pubkey(),
    );
    send_tx_expect_ok(
        &mut env.svm,
        approve_session,
        &[&env.admin, &smart_wallet_owner],
    );

    let execute = client.execute_with_session(
        env.admin.pubkey(),
        session_key.pubkey(),
        user_smart_wallet,
        increment_ix,
    );
    send_tx_expect_ok(&mut env.svm, execute, &[&env.admin, &session_key]);

    let counter: Counter = load(&env, &mock_client::counter_pda());
    assert_eq!(counter.count, 1);
    assert_eq!(counter.authority, user_smart_wallet);
}

#[test]
fn test_update_rejects_non_executor() {
    let client = client::R3SessionKeysClient::new();
    let mut env = Env::new();
    let initialize_ix = client.initialize(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, initialize_ix, &[&env.admin]);

    let smart_wallet_owner = Keypair::new();
    let session_key = Keypair::new();
    let wrong_executor = Keypair::new();
    let (create_wallet, user_smart_wallet, _) =
        client.create_smart_wallet(env.admin.pubkey(), smart_wallet_owner.pubkey());
    send_tx_expect_ok(&mut env.svm, create_wallet, &[&env.admin]);
    let (create_session, _, _) = client.create_session(
        env.admin.pubkey(),
        user_smart_wallet,
        session_key.pubkey(),
        TARGET_PROGRAM_PLACEHOLDER,
        timestamp_from_future(),
        DUMMY_ANCHOR_DISCRIMINATOR.to_vec(),
        8,
    );
    send_tx_expect_ok(&mut env.svm, create_session, &[&env.admin]);

    let update_session = client.update_session(
        wrong_executor.pubkey(),
        user_smart_wallet,
        session_key.pubkey(),
        TARGET_PROGRAM_PLACEHOLDER,
        timestamp_from_future(),
        DUMMY_ANCHOR_DISCRIMINATOR.to_vec(),
        8,
    );
    let error = send_tx_expect_error(&mut env.svm, update_session, &[&env.admin, &wrong_executor]);
    assert!(
        error.contains("Error Code: UnauthorizedSessionExecutor"),
        "{error}"
    );
}

#[test]
fn test_update_rejects_revoked_session() {
    let client = client::R3SessionKeysClient::new();
    let mut env = Env::new();
    let initialize_ix = client.initialize(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, initialize_ix, &[&env.admin]);

    let smart_wallet_owner = Keypair::new();
    let session_key = Keypair::new();
    let (create_wallet, user_smart_wallet, _) =
        client.create_smart_wallet(env.admin.pubkey(), smart_wallet_owner.pubkey());
    send_tx_expect_ok(&mut env.svm, create_wallet, &[&env.admin]);
    let (create_session, _, _) = client.create_session(
        env.admin.pubkey(),
        user_smart_wallet,
        session_key.pubkey(),
        TARGET_PROGRAM_PLACEHOLDER,
        timestamp_from_future(),
        DUMMY_ANCHOR_DISCRIMINATOR.to_vec(),
        8,
    );
    send_tx_expect_ok(&mut env.svm, create_session, &[&env.admin]);

    let revoke_session = client.revoke_session(
        smart_wallet_owner.pubkey(),
        user_smart_wallet,
        session_key.pubkey(),
    );
    send_tx_expect_ok(
        &mut env.svm,
        revoke_session,
        &[&env.admin, &smart_wallet_owner],
    );

    let update_session = client.update_session(
        env.admin.pubkey(),
        user_smart_wallet,
        session_key.pubkey(),
        TARGET_PROGRAM_PLACEHOLDER,
        timestamp_from_future(),
        DUMMY_ANCHOR_DISCRIMINATOR.to_vec(),
        8,
    );
    let error = send_tx_expect_error(&mut env.svm, update_session, &[&env.admin]);
    assert!(
        error.contains("Error Code: InvalidSessionStatus"),
        "{error}"
    );
}
