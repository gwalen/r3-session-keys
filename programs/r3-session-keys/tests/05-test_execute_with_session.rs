mod common;

use {
    anchor_lang::AccountSerialize,
    common::{client, load, mock_client, send_tx_expect_ok, Env},
    mock_client::mock_program::accounts::Counter,
    solana_keypair::Keypair,
    solana_signer::Signer,
    std::time::{SystemTime, UNIX_EPOCH},
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

    // The mock initialize instruction makes its payer the counter authority.
    // LiteSVM changes only that field so the test can exercise PDA signing.
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
fn test_execute_mock_increment_with_session() {
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
    let mock_counter = mock_client::counter_pda();
    let increment_ix = mock_client.increment(user_smart_wallet);
    let increment_discriminator = mock_client::increment_discriminator().to_vec();
    assert!(increment_ix.data.starts_with(&increment_discriminator));

    let expires_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        + 1000;
    let (create_session, _, _) = client.create_session(
        env.admin.pubkey(),
        user_smart_wallet,
        session_key.pubkey(),
        expires_at,
        increment_discriminator.clone(),
        u8::try_from(increment_discriminator.len()).unwrap(),
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

    let execute = client.execute_with_session(
        env.admin.pubkey(),
        session_key.pubkey(),
        user_smart_wallet,
        increment_ix,
    );
    send_tx_expect_ok(&mut env.svm, execute, &[&env.admin, &session_key]);

    let counter: Counter = load(&env, &mock_counter);
    assert_eq!(counter.count, 1);
    assert_eq!(counter.authority, user_smart_wallet);
}
