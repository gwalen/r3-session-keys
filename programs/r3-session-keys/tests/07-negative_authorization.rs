mod common;

use {
    anchor_lang::{prelude::AccountMeta, solana_program::instruction::Instruction},
    common::{
        client::R3SessionKeysClient, load, send_tx_expect_error, send_tx_expect_ok,
        timestamp_from_future, Env, DUMMY_ANCHOR_DISCRIMINATOR, TARGET_PROGRAM_PLACEHOLDER,
    },
    r3_session_keys::state::{
        program_config::{ProgramConfig, ProgramStatus},
        session::{Session, SessionStatus},
    },
    solana_keypair::Keypair,
    solana_signer::Signer,
};

#[test]
fn test_approve_rejects_non_smart_wallet_owner() {
    let client = R3SessionKeysClient::new();
    let mut env = Env::new();
    let initialize_ix = client.initialize(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, initialize_ix, &[&env.admin]);

    let smart_wallet_owner = Keypair::new();
    let session_key = Keypair::new().pubkey();

    let (ix, user_smart_wallet, _) =
        client.create_smart_wallet(env.admin.pubkey(), smart_wallet_owner.pubkey());
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);

    let (ix, session, _) = client.create_session(
        env.admin.pubkey(),
        user_smart_wallet,
        session_key,
        TARGET_PROGRAM_PLACEHOLDER,
        timestamp_from_future(),
        DUMMY_ANCHOR_DISCRIMINATOR.to_vec(),
        8,
    );
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);

    let wrong_owner = Keypair::new();
    let ix = client.approve_session(wrong_owner.pubkey(), user_smart_wallet, session_key);
    let error = send_tx_expect_error(&mut env.svm, ix, &[&env.admin, &wrong_owner]);

    assert!(error.contains("Error Code: ConstraintSeeds"), "{error}");
    let state: Session = load(&env, &session);
    assert_eq!(state.status, SessionStatus::WaitingForApproval);
}

#[test]
fn test_revoke_rejects_non_smart_wallet_owner() {
    let client = R3SessionKeysClient::new();
    let mut env = Env::new();
    let initialize_ix = client.initialize(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, initialize_ix, &[&env.admin]);

    let smart_wallet_owner = Keypair::new();
    let session_key = Keypair::new().pubkey();

    let (ix, user_smart_wallet, _) =
        client.create_smart_wallet(env.admin.pubkey(), smart_wallet_owner.pubkey());
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);

    let (ix, session, _) = client.create_session(
        env.admin.pubkey(),
        user_smart_wallet,
        session_key,
        TARGET_PROGRAM_PLACEHOLDER,
        timestamp_from_future(),
        DUMMY_ANCHOR_DISCRIMINATOR.to_vec(),
        8,
    );
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);

    let wrong_owner = Keypair::new();
    let ix = client.revoke_session(wrong_owner.pubkey(), user_smart_wallet, session_key);
    let error = send_tx_expect_error(&mut env.svm, ix, &[&env.admin, &wrong_owner]);

    assert!(error.contains("Error Code: ConstraintSeeds"), "{error}");
    let state: Session = load(&env, &session);
    assert_eq!(state.status, SessionStatus::WaitingForApproval);
}

#[test]
fn test_execute_rejects_unauthorized_session_executor() {
    let client = R3SessionKeysClient::new();
    let mut env = Env::new();
    let initialize_ix = client.initialize(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, initialize_ix, &[&env.admin]);

    let smart_wallet_owner = Keypair::new();
    let session_key = Keypair::new();

    let (ix, user_smart_wallet, _) =
        client.create_smart_wallet(env.admin.pubkey(), smart_wallet_owner.pubkey());
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);

    let (ix, _, _) = client.create_session(
        env.admin.pubkey(),
        user_smart_wallet,
        session_key.pubkey(),
        TARGET_PROGRAM_PLACEHOLDER,
        timestamp_from_future(),
        DUMMY_ANCHOR_DISCRIMINATOR.to_vec(),
        8,
    );
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);

    let ix = client.approve_session(
        smart_wallet_owner.pubkey(),
        user_smart_wallet,
        session_key.pubkey(),
    );
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin, &smart_wallet_owner]);

    let wrong_executor = Keypair::new();
    let target_ix = Instruction {
        program_id: TARGET_PROGRAM_PLACEHOLDER,
        accounts: vec![AccountMeta::new_readonly(user_smart_wallet, false)],
        data: DUMMY_ANCHOR_DISCRIMINATOR.to_vec(),
    };
    let ix = client.execute_with_session(
        wrong_executor.pubkey(),
        session_key.pubkey(),
        user_smart_wallet,
        target_ix,
    );
    let error = send_tx_expect_error(
        &mut env.svm,
        ix,
        &[&env.admin, &wrong_executor, &session_key],
    );

    assert!(
        error.contains("Error Code: UnauthorizedSessionExecutor"),
        "{error}"
    );
}

#[test]
fn test_pause_rejects_non_admin() {
    let client = R3SessionKeysClient::new();
    let mut env = Env::new();
    let initialize_ix = client.initialize(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, initialize_ix, &[&env.admin]);

    let wrong_admin = Keypair::new();
    let ix = client.pause(wrong_admin.pubkey());
    let error = send_tx_expect_error(&mut env.svm, ix, &[&env.admin, &wrong_admin]);

    assert!(error.contains("Error Code: UnauthorizedAdmin"), "{error}");
    let config: ProgramConfig = load(&env, &env.program_config);
    assert_eq!(config.status, ProgramStatus::Active);
}

#[test]
fn test_unpause_rejects_non_admin() {
    let client = R3SessionKeysClient::new();
    let mut env = Env::new();
    let initialize_ix = client.initialize(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, initialize_ix, &[&env.admin]);

    let ix = client.pause(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);

    let wrong_admin = Keypair::new();
    let ix = client.unpause(wrong_admin.pubkey());
    let error = send_tx_expect_error(&mut env.svm, ix, &[&env.admin, &wrong_admin]);

    assert!(error.contains("Error Code: UnauthorizedAdmin"), "{error}");
    let config: ProgramConfig = load(&env, &env.program_config);
    assert_eq!(config.status, ProgramStatus::Paused);
}
