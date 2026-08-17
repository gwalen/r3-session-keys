mod common;

use {
    anchor_lang::{prelude::AccountMeta, solana_program::instruction::Instruction},
    common::{
        client, load, send_tx_expect_error, send_tx_expect_ok, timestamp_from_future, Env,
        DUMMY_ANCHOR_DISCRIMINATOR, TARGET_PROGRAM_PLACEHOLDER,
    },
    r3_session_keys::state::{
        program_config::{ProgramConfig, ProgramStatus},
        session::{Session, SessionStatus},
    },
    solana_keypair::Keypair,
    solana_signer::Signer,
};

struct SessionFixture {
    smart_wallet_owner: Keypair,
    session_key: Keypair,
    user_smart_wallet: anchor_lang::prelude::Pubkey,
    session: anchor_lang::prelude::Pubkey,
}

fn initialize(env: &mut Env, client: &client::R3SessionKeysClient) {
    let ix = client.initialize(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);
}

fn create_session_fixture(env: &mut Env, client: &client::R3SessionKeysClient) -> SessionFixture {
    let smart_wallet_owner = Keypair::new();
    let session_key = Keypair::new();

    let (ix, user_smart_wallet, _) =
        client.create_smart_wallet(env.admin.pubkey(), smart_wallet_owner.pubkey());
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);

    let (ix, session, _) = client.create_session(
        env.admin.pubkey(),
        user_smart_wallet,
        session_key.pubkey(),
        TARGET_PROGRAM_PLACEHOLDER,
        timestamp_from_future(),
        DUMMY_ANCHOR_DISCRIMINATOR.to_vec(),
        DUMMY_ANCHOR_DISCRIMINATOR.len() as u8,
    );
    send_tx_expect_ok(&mut env.svm, ix, &[&env.admin]);

    SessionFixture {
        smart_wallet_owner,
        session_key,
        user_smart_wallet,
        session,
    }
}

fn assert_error(error: &str, expected_code: &str) {
    assert!(
        error.contains(&format!("Error Code: {expected_code}")),
        "expected {expected_code}, got:\n{error}"
    );
}

#[test]
fn test_approve_rejects_non_smart_wallet_owner() {
    let client = client::R3SessionKeysClient::new();
    let mut env = Env::new();
    initialize(&mut env, &client);
    let fixture = create_session_fixture(&mut env, &client);
    let wrong_owner = Keypair::new();

    let ix = client.approve_session(
        wrong_owner.pubkey(),
        fixture.user_smart_wallet,
        fixture.session_key.pubkey(),
    );
    let error = send_tx_expect_error(&mut env.svm, ix, &[&env.admin, &wrong_owner]);

    assert_error(&error, "ConstraintSeeds");
    let session: Session = load(&env, &fixture.session);
    assert_eq!(session.status, SessionStatus::WaitingForApproval);
}

#[test]
fn test_revoke_rejects_non_smart_wallet_owner() {
    let client = client::R3SessionKeysClient::new();
    let mut env = Env::new();
    initialize(&mut env, &client);
    let fixture = create_session_fixture(&mut env, &client);
    let wrong_owner = Keypair::new();

    let ix = client.revoke_session(
        wrong_owner.pubkey(),
        fixture.user_smart_wallet,
        fixture.session_key.pubkey(),
    );
    let error = send_tx_expect_error(&mut env.svm, ix, &[&env.admin, &wrong_owner]);

    assert_error(&error, "ConstraintSeeds");
    let session: Session = load(&env, &fixture.session);
    assert_eq!(session.status, SessionStatus::WaitingForApproval);
}

#[test]
fn test_execute_rejects_unauthorized_session_executor() {
    let client = client::R3SessionKeysClient::new();
    let mut env = Env::new();
    initialize(&mut env, &client);
    let fixture = create_session_fixture(&mut env, &client);

    let approve = client.approve_session(
        fixture.smart_wallet_owner.pubkey(),
        fixture.user_smart_wallet,
        fixture.session_key.pubkey(),
    );
    send_tx_expect_ok(
        &mut env.svm,
        approve,
        &[&env.admin, &fixture.smart_wallet_owner],
    );

    let wrong_executor = Keypair::new();
    let target_ix = Instruction {
        program_id: TARGET_PROGRAM_PLACEHOLDER,
        accounts: vec![AccountMeta::new_readonly(fixture.user_smart_wallet, false)],
        data: DUMMY_ANCHOR_DISCRIMINATOR.to_vec(),
    };
    let execute = client.execute_with_session(
        wrong_executor.pubkey(),
        fixture.session_key.pubkey(),
        fixture.user_smart_wallet,
        target_ix,
    );
    let error = send_tx_expect_error(
        &mut env.svm,
        execute,
        &[&env.admin, &wrong_executor, &fixture.session_key],
    );

    assert_error(&error, "UnauthorizedSessionExecutor");
}

#[test]
fn test_pause_rejects_non_admin() {
    let client = client::R3SessionKeysClient::new();
    let mut env = Env::new();
    initialize(&mut env, &client);
    let wrong_admin = Keypair::new();

    let ix = client.pause(wrong_admin.pubkey());
    let error = send_tx_expect_error(&mut env.svm, ix, &[&env.admin, &wrong_admin]);

    assert_error(&error, "UnauthorizedAdmin");
    let config: ProgramConfig = load(&env, &env.program_config);
    assert_eq!(config.status, ProgramStatus::Active);
}

#[test]
fn test_unpause_rejects_non_admin() {
    let client = client::R3SessionKeysClient::new();
    let mut env = Env::new();
    initialize(&mut env, &client);

    let pause = client.pause(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, pause, &[&env.admin]);

    let wrong_admin = Keypair::new();
    let unpause = client.unpause(wrong_admin.pubkey());
    let error = send_tx_expect_error(&mut env.svm, unpause, &[&env.admin, &wrong_admin]);

    assert_error(&error, "UnauthorizedAdmin");
    let config: ProgramConfig = load(&env, &env.program_config);
    assert_eq!(config.status, ProgramStatus::Paused);
}
