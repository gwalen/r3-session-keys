mod common;

use {
    anchor_lang::{
        prelude::{AccountMeta, Pubkey},
        solana_program::instruction::Instruction,
    },
    common::{
        client::R3SessionKeysClient, send_tx_expect_error, send_tx_expect_ok,
        timestamp_from_future, Env, DUMMY_ANCHOR_DISCRIMINATOR, TARGET_PROGRAM_PLACEHOLDER,
    },
    solana_keypair::Keypair,
    solana_signer::Signer,
};

fn create_approved_session(
    env: &mut Env,
    client: &R3SessionKeysClient,
) -> (Keypair, Pubkey, Pubkey) {
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

    (session_key, user_smart_wallet, session)
}

fn execute_with_remaining_accounts(
    env: &mut Env,
    client: &R3SessionKeysClient,
    session_key: &Keypair,
    user_smart_wallet: Pubkey,
    accounts: Vec<AccountMeta>,
) -> String {
    let target_ix = Instruction {
        program_id: TARGET_PROGRAM_PLACEHOLDER,
        accounts,
        data: DUMMY_ANCHOR_DISCRIMINATOR.to_vec(),
    };
    let execute = client.execute_with_session(
        env.admin.pubkey(),
        session_key.pubkey(),
        user_smart_wallet,
        target_ix,
    );
    send_tx_expect_error(&mut env.svm, execute, &[&env.admin, session_key])
}

#[test]
fn test_execute_rejects_session_key_in_remaining_accounts() {
    let client = R3SessionKeysClient::new();
    let mut env = Env::new();
    let initialize_ix = client.initialize(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, initialize_ix, &[&env.admin]);

    let (session_key, user_smart_wallet, _) = create_approved_session(&mut env, &client);

    let error = execute_with_remaining_accounts(
        &mut env,
        &client,
        &session_key,
        user_smart_wallet,
        vec![
            AccountMeta::new_readonly(session_key.pubkey(), false),
            AccountMeta::new_readonly(user_smart_wallet, false),
        ],
    );

    assert!(
        error.contains("Error Code: RemainingAccountsContainsSessionKey"),
        "{error}"
    );
}

#[test]
fn test_execute_rejects_other_program_owned_account() {
    let client = R3SessionKeysClient::new();
    let mut env = Env::new();
    let initialize_ix = client.initialize(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, initialize_ix, &[&env.admin]);

    let (session_key, user_smart_wallet, session) = create_approved_session(&mut env, &client);

    let error = execute_with_remaining_accounts(
        &mut env,
        &client,
        &session_key,
        user_smart_wallet,
        vec![
            AccountMeta::new_readonly(session, false),
            AccountMeta::new_readonly(user_smart_wallet, false),
        ],
    );

    assert!(
        error.contains("Error Code: RemainingAccountsContainsProgramOwnedAccount"),
        "{error}"
    );
}

#[test]
fn test_execute_rejects_writable_smart_wallet() {
    let client = R3SessionKeysClient::new();
    let mut env = Env::new();
    let initialize_ix = client.initialize(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, initialize_ix, &[&env.admin]);

    let (session_key, user_smart_wallet, _) = create_approved_session(&mut env, &client);

    let error = execute_with_remaining_accounts(
        &mut env,
        &client,
        &session_key,
        user_smart_wallet,
        vec![AccountMeta::new(user_smart_wallet, false)],
    );

    assert!(
        error.contains("Error Code: UserSmartWalletAccountIsWritable"),
        "{error}"
    );
}

#[test]
fn test_execute_rejects_missing_smart_wallet() {
    let client = R3SessionKeysClient::new();
    let mut env = Env::new();
    let initialize_ix = client.initialize(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, initialize_ix, &[&env.admin]);

    let (session_key, user_smart_wallet, _) = create_approved_session(&mut env, &client);

    let error =
        execute_with_remaining_accounts(&mut env, &client, &session_key, user_smart_wallet, vec![]);

    assert!(
        error.contains("Error Code: UserSmartWalletNotFound"),
        "{error}"
    );
}

#[test]
fn test_execute_rejects_duplicate_smart_wallet() {
    let client = R3SessionKeysClient::new();
    let mut env = Env::new();
    let initialize_ix = client.initialize(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, initialize_ix, &[&env.admin]);

    let (session_key, user_smart_wallet, _) = create_approved_session(&mut env, &client);

    let error = execute_with_remaining_accounts(
        &mut env,
        &client,
        &session_key,
        user_smart_wallet,
        vec![
            AccountMeta::new_readonly(user_smart_wallet, false),
            AccountMeta::new_readonly(user_smart_wallet, false),
        ],
    );

    assert!(
        error.contains("Error Code: MultipleUserSmartWalletAccounts"),
        "{error}"
    );
}
