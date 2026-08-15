mod common;

use {
    common::{client, load, send, Env},
    r3_session_keys::state::{
        session::{Session, SessionStatus},
        user_smart_wallet::UserSmartWallet,
    },
    solana_keypair::Keypair,
    solana_signer::Signer,
};

#[test]
fn test_create_smart_wallet() {
    let mut env = Env::new();
    let user_wallet = Keypair::new().pubkey();
    let (ix, user_smart_wallet, bump) = client::create_smart_wallet(env.admin.pubkey(), user_wallet);

    send(&mut env, ix);

    let state: UserSmartWallet = load(&env, &user_smart_wallet);
    assert_eq!(state.owner, user_wallet);
    assert_eq!(state.bump, bump);
}

#[test]
fn test_create_session() {
    let mut env = Env::new();
    let user_wallet = Keypair::new().pubkey();
    let session_key = Keypair::new().pubkey();

    let (ix, user_smart_wallet, _) = client::create_smart_wallet(env.admin.pubkey(), user_wallet);
    send(&mut env, ix);

    let (ix, session, bump) =
        client::create_session(env.admin.pubkey(), user_smart_wallet, session_key);
    send(&mut env, ix);

    let state: Session = load(&env, &session);
    assert_eq!(state.session_owner, env.admin.pubkey());
    assert_eq!(state.session_key, session_key);
    assert_eq!(state.expires_at, 0);
    assert!(state.allowed_writeable_mint_list.is_empty());
    assert!(state.mint_limits.is_empty());
    assert!(state.status == SessionStatus::WaitingForApproval);
    assert_eq!(state.nonce, 0);
    assert_eq!(state.bump, bump);
}
