mod common;

use {
    anchor_lang::{
        prelude::{AccountMeta, Pubkey},
        solana_program::instruction::Instruction,
    },
    common::{
        client, send_tx_expect_error, send_tx_expect_ok, timestamp_from_future, Env,
        DUMMY_ANCHOR_DISCRIMINATOR, TARGET_PROGRAM_PLACEHOLDER,
    },
    solana_keypair::Keypair,
    solana_signer::Signer,
};

struct ApprovedSessionFixture {
    session_key: Keypair,
    user_smart_wallet: Pubkey,
    session: Pubkey,
}

fn create_approved_session(
    env: &mut Env,
    client: &client::R3SessionKeysClient,
) -> ApprovedSessionFixture {
    let initialize = client.initialize(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, initialize, &[&env.admin]);

    let smart_wallet_owner = Keypair::new();
    let session_key = Keypair::new();
    let (create_wallet, user_smart_wallet, _) =
        client.create_smart_wallet(env.admin.pubkey(), smart_wallet_owner.pubkey());
    send_tx_expect_ok(&mut env.svm, create_wallet, &[&env.admin]);

    let (create_session, session, _) = client.create_session(
        env.admin.pubkey(),
        user_smart_wallet,
        session_key.pubkey(),
        TARGET_PROGRAM_PLACEHOLDER,
        timestamp_from_future(),
        DUMMY_ANCHOR_DISCRIMINATOR.to_vec(),
        DUMMY_ANCHOR_DISCRIMINATOR.len() as u8,
    );
    send_tx_expect_ok(&mut env.svm, create_session, &[&env.admin]);

    let approve = client.approve_session(
        smart_wallet_owner.pubkey(),
        user_smart_wallet,
        session_key.pubkey(),
    );
    send_tx_expect_ok(&mut env.svm, approve, &[&env.admin, &smart_wallet_owner]);

    ApprovedSessionFixture {
        session_key,
        user_smart_wallet,
        session,
    }
}

fn execute_with_remaining_accounts(
    env: &mut Env,
    client: &client::R3SessionKeysClient,
    fixture: &ApprovedSessionFixture,
    accounts: Vec<AccountMeta>,
) -> String {
    let target_ix = Instruction {
        program_id: TARGET_PROGRAM_PLACEHOLDER,
        accounts,
        data: DUMMY_ANCHOR_DISCRIMINATOR.to_vec(),
    };
    let execute = client.execute_with_session(
        env.admin.pubkey(),
        fixture.session_key.pubkey(),
        fixture.user_smart_wallet,
        target_ix,
    );
    send_tx_expect_error(&mut env.svm, execute, &[&env.admin, &fixture.session_key])
}

fn assert_error(error: &str, expected_code: &str) {
    assert!(
        error.contains(&format!("Error Code: {expected_code}")),
        "expected {expected_code}, got:\n{error}"
    );
}

#[test]
fn test_execute_rejects_session_key_in_remaining_accounts() {
    let client = client::R3SessionKeysClient::new();
    let mut env = Env::new();
    let fixture = create_approved_session(&mut env, &client);

    let error = execute_with_remaining_accounts(
        &mut env,
        &client,
        &fixture,
        vec![
            AccountMeta::new_readonly(fixture.session_key.pubkey(), false),
            AccountMeta::new_readonly(fixture.user_smart_wallet, false),
        ],
    );

    assert_error(&error, "RemainingAccountsContainsSessionKey");
}

#[test]
fn test_execute_rejects_other_program_owned_account() {
    let client = client::R3SessionKeysClient::new();
    let mut env = Env::new();
    let fixture = create_approved_session(&mut env, &client);

    let error = execute_with_remaining_accounts(
        &mut env,
        &client,
        &fixture,
        vec![
            AccountMeta::new_readonly(fixture.session, false),
            AccountMeta::new_readonly(fixture.user_smart_wallet, false),
        ],
    );

    assert_error(&error, "RemainingAccountsContainsProgramOwnedAccount");
}

#[test]
fn test_execute_rejects_writable_smart_wallet() {
    let client = client::R3SessionKeysClient::new();
    let mut env = Env::new();
    let fixture = create_approved_session(&mut env, &client);

    let error = execute_with_remaining_accounts(
        &mut env,
        &client,
        &fixture,
        vec![AccountMeta::new(fixture.user_smart_wallet, false)],
    );

    assert_error(&error, "UserSmartWalletAccountIsWritable");
}

#[test]
fn test_execute_rejects_missing_smart_wallet() {
    let client = client::R3SessionKeysClient::new();
    let mut env = Env::new();
    let fixture = create_approved_session(&mut env, &client);

    let error = execute_with_remaining_accounts(&mut env, &client, &fixture, vec![]);

    assert_error(&error, "UserSmartWalletNotFound");
}

#[test]
fn test_execute_rejects_duplicate_smart_wallet() {
    let client = client::R3SessionKeysClient::new();
    let mut env = Env::new();
    let fixture = create_approved_session(&mut env, &client);

    let error = execute_with_remaining_accounts(
        &mut env,
        &client,
        &fixture,
        vec![
            AccountMeta::new_readonly(fixture.user_smart_wallet, false),
            AccountMeta::new_readonly(fixture.user_smart_wallet, false),
        ],
    );

    assert_error(&error, "MultipleUserSmartWalletAccounts");
}
