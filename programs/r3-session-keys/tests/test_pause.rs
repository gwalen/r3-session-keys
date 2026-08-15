mod common;

use {
    common::{assert_program_paused, client, load, send, send_err, Env},
    r3_session_keys::state::{
        program_config::{ProgramConfig, ProgramStatus},
        session::Session,
        user_smart_wallet::UserSmartWallet,
    },
    solana_keypair::Keypair,
    solana_signer::Signer,
};

#[test]
fn test_pause_blocks_creates_and_unpause_restores() {
    let mut env = Env::new();
    let user_wallet = Keypair::new().pubkey();
    let other_user_wallet = Keypair::new().pubkey();
    let session_key = Keypair::new().pubkey();

    let (ix, user_smart_wallet, _) = client::create_smart_wallet(env.admin.pubkey(), user_wallet);
    send(&mut env, ix);

    let ix = client::pause(env.admin.pubkey());
    send(&mut env, ix);
    let config: ProgramConfig = load(&env, &env.program_config);
    assert!(config.status == ProgramStatus::Paused);

    let (ix, _, _) = client::create_smart_wallet(env.admin.pubkey(), other_user_wallet);
    assert_program_paused(&send_err(&mut env, ix));

    let (ix, _, _) = client::create_session(env.admin.pubkey(), user_smart_wallet, session_key);
    assert_program_paused(&send_err(&mut env, ix));

    let ix = client::unpause(env.admin.pubkey());
    send(&mut env, ix);
    let config: ProgramConfig = load(&env, &env.program_config);
    assert!(config.status == ProgramStatus::Active);

    let (ix, other_smart_wallet, _) = client::create_smart_wallet(env.admin.pubkey(), other_user_wallet);
    send(&mut env, ix);
    let _: UserSmartWallet = load(&env, &other_smart_wallet);

    let (ix, session, _) = client::create_session(env.admin.pubkey(), user_smart_wallet, session_key);
    send(&mut env, ix);
    let _: Session = load(&env, &session);
}
