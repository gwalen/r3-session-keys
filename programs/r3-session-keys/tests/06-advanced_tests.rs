mod common;

use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{program_pack::Pack, system_instruction},
        AccountDeserialize,
    },
    anchor_spl::{
        associated_token,
        token::{self, spl_token},
        token_2022,
        token_interface::{Mint, TokenAccount},
    },
    common::{client, mock_client, send_tx_expect_error, send_tx_expect_ok, Env},
    solana_keypair::Keypair,
    solana_signer::Signer,
    std::time::{SystemTime, UNIX_EPOCH},
    associated_token::spl_associated_token_account::instruction::create_associated_token_account,
};

const TOKEN_DECIMALS: u8 = 6;
const INITIAL_TOKEN_BALANCE: u64 = 1_000_000;
const DEPOSIT_AMOUNT: u64 = 250_000;

struct TokenPoolFixture {
    deposit_mint: Pubkey,
    lp_mint: Pubkey,
    vault: Pubkey,
    user_deposit_account: Pubkey,
    user_lp_account: Pubkey,
}

fn load_mock_program(env: &mut Env) {
    let bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/mock-program/mock_program.so"
    ));
    env.svm
        .add_program(mock_client::mock_program::ID, bytes)
        .unwrap();
}

fn initialize_mock_token_pool(
    env: &mut Env,
    mock_client: &mock_client::MockClient,
    token_owner: Pubkey,
) -> TokenPoolFixture {
    let deposit_mint = Keypair::new();
    let mint_len = spl_token::state::Mint::LEN;
    let create_mint = system_instruction::create_account(
        &env.admin.pubkey(),
        &deposit_mint.pubkey(),
        env.svm.minimum_balance_for_rent_exemption(mint_len),
        mint_len as u64,
        &token::ID,
    );
    send_tx_expect_ok(&mut env.svm, create_mint, &[&env.admin, &deposit_mint]);

    let initialize_mint = spl_token::instruction::initialize_mint2(
        &token::ID,
        &deposit_mint.pubkey(),
        &env.admin.pubkey(),
        None,
        TOKEN_DECIMALS,
    )
    .unwrap();
    send_tx_expect_ok(&mut env.svm, initialize_mint, &[&env.admin]);

    let user_deposit_account =
        mock_client::user_deposit_account(&token_owner, &deposit_mint.pubkey());
    let create_user_deposit_account = create_associated_token_account(
        &env.admin.pubkey(),
        &token_owner,
        &deposit_mint.pubkey(),
        &token::ID,
    );
    send_tx_expect_ok(&mut env.svm, create_user_deposit_account, &[&env.admin]);

    let mint_deposit_tokens = spl_token::instruction::mint_to(
        &token::ID,
        &deposit_mint.pubkey(),
        &user_deposit_account,
        &env.admin.pubkey(),
        &[],
        INITIAL_TOKEN_BALANCE,
    )
    .unwrap();
    send_tx_expect_ok(&mut env.svm, mint_deposit_tokens, &[&env.admin]);

    let initialize_pool = mock_client.initialize_pool(env.admin.pubkey(), deposit_mint.pubkey());
    send_tx_expect_ok(&mut env.svm, initialize_pool, &[&env.admin]);

    let pool = mock_client::pool_pda(&deposit_mint.pubkey());
    let lp_mint = mock_client::lp_mint_pda(&pool);
    let vault = mock_client::vault_address(&pool, &deposit_mint.pubkey());
    let user_lp_account = mock_client::user_lp_account(&token_owner, &lp_mint);
    let create_user_lp_account = create_associated_token_account(
        &env.admin.pubkey(),
        &token_owner,
        &lp_mint,
        &token_2022::ID,
    );
    send_tx_expect_ok(&mut env.svm, create_user_lp_account, &[&env.admin]);

    TokenPoolFixture {
        deposit_mint: deposit_mint.pubkey(),
        lp_mint,
        vault,
        user_deposit_account,
        user_lp_account,
    }
}

fn create_approved_session(
    env: &mut Env,
    client: &client::R3SessionKeysClient,
    allowed_discriminators: Vec<u8>,
    discriminator_size: u8,
) -> (Keypair, Pubkey) {
    let smart_wallet_owner = Keypair::new();
    let session_key = Keypair::new();
    let (create_wallet, user_smart_wallet, _) =
        client.create_smart_wallet(env.admin.pubkey(), smart_wallet_owner.pubkey());
    send_tx_expect_ok(&mut env.svm, create_wallet, &[&env.admin]);

    let expires_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        + 1000;
    let (create_session, _, _) = client.create_session(
        env.admin.pubkey(),
        user_smart_wallet,
        session_key.pubkey(),
        mock_client::mock_program::ID,
        expires_at,
        allowed_discriminators,
        discriminator_size,
    );
    send_tx_expect_ok(&mut env.svm, create_session, &[&env.admin]);

    let approve_session = client.approve_session(
        smart_wallet_owner.pubkey(),
        user_smart_wallet,
        session_key.pubkey(),
    );
    send_tx_expect_ok(
        &mut env.svm,
        approve_session,
        &[&env.admin, &smart_wallet_owner],
    );

    (session_key, user_smart_wallet)
}

fn token_account(env: &Env, address: &Pubkey) -> TokenAccount {
    let account = env.svm.get_account(address).unwrap();
    let mut data: &[u8] = &account.data;
    TokenAccount::try_deserialize(&mut data).unwrap()
}

fn mint_supply(env: &Env, address: &Pubkey) -> u64 {
    let account = env.svm.get_account(address).unwrap();
    let mut data: &[u8] = &account.data;
    Mint::try_deserialize(&mut data).unwrap().supply
}

#[test]
fn test_execute_mock_deposit_with_smart_wallet_tokens() {
    let client = client::R3SessionKeysClient::new();
    let mut env = Env::new();
    let initialize_ix = client.initialize(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, initialize_ix, &[&env.admin]);

    load_mock_program(&mut env);
    let mock_client = mock_client::MockClient::new();
    let deposit_discriminator = mock_client::deposit_discriminator().to_vec();
    let (session_key, user_smart_wallet) = create_approved_session(
        &mut env,
        &client,
        deposit_discriminator.clone(),
        u8::try_from(deposit_discriminator.len()).unwrap(),
    );

    let session_executor = env.admin.pubkey();
    let token_pool_fixture = initialize_mock_token_pool(&mut env, &mock_client, user_smart_wallet);
    let deposit_account_before = token_account(&env, &token_pool_fixture.user_deposit_account);
    assert_eq!(deposit_account_before.owner, user_smart_wallet);
    assert_eq!(deposit_account_before.amount, INITIAL_TOKEN_BALANCE);

    // The smart wallet is the mock-program user. execute_with_session strips its
    // is_signer flag for the outer tx and restores it on the CPI via invoke_signed.
    let deposit_ix =
        mock_client.deposit(user_smart_wallet, token_pool_fixture.deposit_mint, DEPOSIT_AMOUNT);
    assert!(deposit_ix.data.starts_with(&deposit_discriminator));
    let execute = client.execute_with_session(
        session_executor,
        session_key.pubkey(),
        user_smart_wallet,
        deposit_ix,
    );
    send_tx_expect_ok(&mut env.svm, execute, &[&env.admin, &session_key]);

    assert_eq!(
        token_account(&env, &token_pool_fixture.user_deposit_account).amount,
        INITIAL_TOKEN_BALANCE - DEPOSIT_AMOUNT
    );
    assert_eq!(
        token_account(&env, &token_pool_fixture.vault).amount,
        DEPOSIT_AMOUNT
    );
    assert_eq!(
        token_account(&env, &token_pool_fixture.user_lp_account).amount,
        DEPOSIT_AMOUNT
    );
    assert_eq!(mint_supply(&env, &token_pool_fixture.lp_mint), DEPOSIT_AMOUNT);
}

#[test]
fn test_execute_mock_withdraw_not_allowed_by_deposit_only_session() {
    let client = client::R3SessionKeysClient::new();
    let mut env = Env::new();
    let initialize_ix = client.initialize(env.admin.pubkey());
    send_tx_expect_ok(&mut env.svm, initialize_ix, &[&env.admin]);

    load_mock_program(&mut env);
    let mock_client = mock_client::MockClient::new();
    let deposit_discriminator = mock_client::deposit_discriminator().to_vec();
    let (session_key, user_smart_wallet) = create_approved_session(
        &mut env,
        &client,
        deposit_discriminator.clone(),
        u8::try_from(deposit_discriminator.len()).unwrap(),
    );

    let session_executor = env.admin.pubkey();
    let token_pool_fixture = initialize_mock_token_pool(&mut env, &mock_client, user_smart_wallet);
    let deposit_ix =
        mock_client.deposit(user_smart_wallet, token_pool_fixture.deposit_mint, DEPOSIT_AMOUNT);
    let execute_deposit = client.execute_with_session(
        session_executor,
        session_key.pubkey(),
        user_smart_wallet,
        deposit_ix,
    );
    send_tx_expect_ok(&mut env.svm, execute_deposit, &[&env.admin, &session_key]);

    let balances_before = (
        token_account(&env, &token_pool_fixture.user_deposit_account).amount,
        token_account(&env, &token_pool_fixture.vault).amount,
        token_account(&env, &token_pool_fixture.user_lp_account).amount,
        mint_supply(&env, &token_pool_fixture.lp_mint),
    );

    let withdraw_discriminator = mock_client::withdraw_discriminator();
    let withdraw_ix =
        mock_client.withdraw(user_smart_wallet, token_pool_fixture.deposit_mint, DEPOSIT_AMOUNT);
    assert!(withdraw_ix.data.starts_with(withdraw_discriminator));
    assert!(!withdraw_ix.data.starts_with(&deposit_discriminator));
    let execute_withdraw = client.execute_with_session(
        session_executor,
        session_key.pubkey(),
        user_smart_wallet,
        withdraw_ix,
    );
    let error = send_tx_expect_error(&mut env.svm, execute_withdraw, &[&env.admin, &session_key]);

    assert!(
        error.contains("Error Code: NotAllowedInstructionDiscriminator"),
        "{error}"
    );
    let balances_after = (
        token_account(&env, &token_pool_fixture.user_deposit_account).amount,
        token_account(&env, &token_pool_fixture.vault).amount,
        token_account(&env, &token_pool_fixture.user_lp_account).amount,
        mint_supply(&env, &token_pool_fixture.lp_mint),
    );
    assert_eq!(balances_after, balances_before);
}
