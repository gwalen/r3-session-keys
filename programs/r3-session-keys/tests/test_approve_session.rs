mod common;

use {
    common::{client, load, send_tx_expect_ok, Env},
    r3_session_keys::state::session::{Session, SessionStatus},
    solana_keypair::Keypair,
    solana_signer::Signer,
    std::time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn test_approve_session_changes_status() {
    let mut env = Env::new();
    let user_wallet = Keypair::new().pubkey();
    let session_key = Keypair::new().pubkey();
    let expires_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        + 1000;

    let (ix, user_smart_wallet, _) = client::create_smart_wallet(env.admin.pubkey(), user_wallet);
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);

    let (ix, session, _) =
        client::create_session(env.admin.pubkey(), user_smart_wallet, session_key, expires_at);
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);

    let state: Session = load(&env, &session);
    assert_eq!(state.status, SessionStatus::WaitingForApproval);

    let ix = client::approve_session(env.admin.pubkey(), user_smart_wallet, session_key);
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);

    let state: Session = load(&env, &session);
    assert_eq!(state.status, SessionStatus::Approved);
}
