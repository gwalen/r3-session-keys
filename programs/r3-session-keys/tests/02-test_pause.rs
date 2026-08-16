mod common;

use {
    common::{assert_program_paused, client, load, send_tx_expect_error, send_tx_expect_ok, Env},
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
    let client = client::R3SessionKeysClient::new();
    let mut env = Env::new();
    let initialize_ix = client.initialize(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, initialize_ix, &[&env.admin]);

    let user_wallet = Keypair::new().pubkey();
    let other_user_wallet = Keypair::new().pubkey();
    let session_key = Keypair::new().pubkey();

    let (ix, user_smart_wallet, _) = client.create_smart_wallet(env.admin.pubkey(), user_wallet);
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);

    let ix = client.pause(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);
    assert_program_paused(&env);

    // program paused so other program functions should fail

    let (ix, _, _) = client.create_smart_wallet(env.admin.pubkey(), other_user_wallet);
    send_tx_expect_error(&mut env.svm, ix, &[&env.admin]);
    assert_program_paused(&env);

    let (ix, _, _) = client.create_session(
        env.admin.pubkey(),
        user_smart_wallet,
        session_key,
        anchor_lang::system_program::ID,
        0,
        vec![],
        8,
    );
    send_tx_expect_error(&mut env.svm, ix, &[&env.admin]);
    assert_program_paused(&env);

    let ix = client.unpause(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);
    let config: ProgramConfig = load(&env, &env.program_config);
    assert!(config.status == ProgramStatus::Active);

    // program unpaused so other program functions should succeed

    let (ix, other_smart_wallet, _) =
        client.create_smart_wallet(env.admin.pubkey(), other_user_wallet);
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);
    let _: UserSmartWallet = load(&env, &other_smart_wallet);

    let (ix, session, _) = client.create_session(
        env.admin.pubkey(),
        user_smart_wallet,
        session_key,
        anchor_lang::system_program::ID,
        0,
        vec![],
        8,
    );
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);
    let _: Session = load(&env, &session);
}
