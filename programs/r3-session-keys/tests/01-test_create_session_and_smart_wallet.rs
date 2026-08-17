mod common;

use {
    common::{
        client, load, send_tx_expect_error, send_tx_expect_ok, Env, DUMMY_ANCHOR_DISCRIMINATOR,
        TARGET_PROGRAM_PLACEHOLDER,
    },
    r3_session_keys::state::{
        session::{Session, SessionStatus},
        user_smart_wallet::UserSmartWallet,
    },
    solana_keypair::Keypair,
    solana_signer::Signer,
    std::time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn test_create_smart_wallet() {
    let client = client::R3SessionKeysClient::new();
    let mut env = Env::new();
    let initialize_ix = client.initialize(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, initialize_ix, &[&env.admin]);

    let user_wallet = Keypair::new().pubkey();
    let (ix, user_smart_wallet, bump) = client.create_smart_wallet(env.admin.pubkey(), user_wallet);

    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);

    let state: UserSmartWallet = load(&env, &user_smart_wallet);
    assert_eq!(state.smart_wallet_owner, user_wallet);
    assert_eq!(state.bump, bump);
}

#[test]
fn test_create_session() {
    let client = client::R3SessionKeysClient::new();
    let mut env = Env::new();
    let initialize_ix = client.initialize(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, initialize_ix, &[&env.admin]);

    let user_wallet = Keypair::new().pubkey();
    let session_key = Keypair::new().pubkey();
    let expires_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        + 1000;

    // Two Anchor-style 8-byte instruction discriminators concatenated.
    let discriminator_len = 8u8;

    let (ix, user_smart_wallet, _) = client.create_smart_wallet(env.admin.pubkey(), user_wallet);
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);

    let (ix, session, bump) = client.create_session(
        env.admin.pubkey(),
        user_smart_wallet,
        session_key,
        TARGET_PROGRAM_PLACEHOLDER,
        expires_at,
        DUMMY_ANCHOR_DISCRIMINATOR.to_vec(),
        discriminator_len,
    );
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);

    let state: Session = load(&env, &session);
    assert_eq!(state.session_executor, env.admin.pubkey());
    assert_eq!(state.session_key, session_key);
    assert_eq!(state.target_program, TARGET_PROGRAM_PLACEHOLDER);
    assert_eq!(state.expires_at, expires_at);
    // assert!(state.allowed_writeable_mint_list.is_empty());
    // assert!(state.mint_limits.is_empty());
    assert_eq!(
        state.allowed_instructions_discriminators,
        DUMMY_ANCHOR_DISCRIMINATOR.to_vec()
    );
    assert_eq!(state.discriminator_size, discriminator_len);
    assert!(state.status == SessionStatus::WaitingForApproval);
    assert_eq!(state.bump, bump);
}

#[test]
fn test_create_session_rejects_invalid_inputs() {
    let client = client::R3SessionKeysClient::new();
    let mut env = Env::new();
    let initialize_ix = client.initialize(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, initialize_ix, &[&env.admin]);

    let user_wallet = Keypair::new().pubkey();
    let (ix, user_smart_wallet, _) = client.create_smart_wallet(env.admin.pubkey(), user_wallet);
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);

    let future_expires_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        + 1000;
    let discriminator = DUMMY_ANCHOR_DISCRIMINATOR.to_vec();

    // discriminator_len = 0 would divide by zero in Session::parse_discriminators
    let (ix, _, _) = client.create_session(
        env.admin.pubkey(),
        user_smart_wallet,
        Keypair::new().pubkey(),
        TARGET_PROGRAM_PLACEHOLDER,
        future_expires_at,
        discriminator.clone(),
        0,
    );
    let error = send_tx_expect_error(&mut env.svm, ix, &[&env.admin]);
    assert!(error.contains("Error Code: InvalidDiscriminatorSize"), "{error}");

    // discriminator list must be a non-empty multiple of the discriminator size
    let (ix, _, _) = client.create_session(
        env.admin.pubkey(),
        user_smart_wallet,
        Keypair::new().pubkey(),
        TARGET_PROGRAM_PLACEHOLDER,
        future_expires_at,
        vec![],
        8,
    );
    let error = send_tx_expect_error(&mut env.svm, ix, &[&env.admin]);
    assert!(error.contains("Error Code: InvalidDiscriminatorListLength"), "{error}");

    let (ix, _, _) = client.create_session(
        env.admin.pubkey(),
        user_smart_wallet,
        Keypair::new().pubkey(),
        TARGET_PROGRAM_PLACEHOLDER,
        future_expires_at,
        vec![0x11, 0x22, 0x33],
        8,
    );
    let error = send_tx_expect_error(&mut env.svm, ix, &[&env.admin]);
    assert!(error.contains("Error Code: InvalidDiscriminatorListLength"), "{error}");

    // expiration must be in the future
    let (ix, _, _) = client.create_session(
        env.admin.pubkey(),
        user_smart_wallet,
        Keypair::new().pubkey(),
        TARGET_PROGRAM_PLACEHOLDER,
        0,
        discriminator,
        8,
    );
    let error = send_tx_expect_error(&mut env.svm, ix, &[&env.admin]);
    assert!(error.contains("Error Code: SessionExpirationInPast"), "{error}");
}
