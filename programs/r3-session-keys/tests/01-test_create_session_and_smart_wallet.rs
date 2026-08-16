mod common;

use {
    common::{client, load, send_tx_expect_ok, Env},
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
    let mut env = Env::new(&client);
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
    let mut env = Env::new(&client);
    let user_wallet = Keypair::new().pubkey();
    let session_key = Keypair::new().pubkey();
    let expires_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        + 1000;

    // Two Anchor-style 8-byte instruction discriminators concatenated.
    let discriminator_len = 8u8;
    let allowed_instructions_discriminators = vec![
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x01,
        0x02,
    ];

    let (ix, user_smart_wallet, _) = client.create_smart_wallet(env.admin.pubkey(), user_wallet);
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);

    let (ix, session, bump) = client.create_session(
        env.admin.pubkey(),
        user_smart_wallet,
        session_key,
        expires_at,
        allowed_instructions_discriminators.clone(),
        discriminator_len,
    );
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);

    let state: Session = load(&env, &session);
    assert_eq!(state.session_executor, env.admin.pubkey());
    assert_eq!(state.session_key, session_key);
    assert_eq!(state.expires_at, expires_at);
    // assert!(state.allowed_writeable_mint_list.is_empty());
    // assert!(state.mint_limits.is_empty());
    assert_eq!(
        state.allowed_instructions_discriminators,
        allowed_instructions_discriminators
    );
    assert_eq!(state.discriminator_size, discriminator_len);
    assert!(state.status == SessionStatus::WaitingForApproval);
    assert_eq!(state.nonce, 0);
    assert_eq!(state.bump, bump);
}
