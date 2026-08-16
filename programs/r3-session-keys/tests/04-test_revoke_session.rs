mod common;

use {
    common::{client, load, send_tx_expect_ok, Env},
    r3_session_keys::state::session::{Session, SessionStatus},
    solana_keypair::Keypair,
    solana_signer::Signer,
    std::time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn test_revoke_session_changes_status() {
    let client = client::R3SessionKeysClient::new();
    let mut env = Env::new();
    let initialize_ix = client.initialize(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, initialize_ix, &[&env.admin]);

    let user = Keypair::new();
    let user_wallet_address = user.pubkey();
    let session_key = Keypair::new().pubkey();
    let expires_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        + 1000;

    let (ix, user_smart_wallet, _) =
        client.create_smart_wallet(env.admin.pubkey(), user_wallet_address);
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);

    let (ix, session, _) = client.create_session(
        env.admin.pubkey(),
        user_smart_wallet,
        session_key,
        expires_at,
        vec![],
        8,
    );
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);

    let state: Session = load(&env, &session);
    assert_eq!(state.status, SessionStatus::WaitingForApproval);

    let ix = client.revoke_session(user.pubkey(), user_smart_wallet, session_key);
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin, &user]);

    let state: Session = load(&env, &session);
    assert_eq!(state.status, SessionStatus::Revoked);
}
