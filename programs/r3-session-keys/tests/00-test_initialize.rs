mod common;

use {
    anchor_lang::AccountDeserialize,
    common::{client::R3SessionKeysClient, send_tx_expect_ok},
    litesvm::LiteSVM,
    r3_session_keys::state::program_config::{ProgramConfig, ProgramStatus},
    solana_keypair::Keypair,
    solana_signer::Signer,
};

#[test]
fn test_initialize() {
    let program_id = r3_session_keys::id();
    let payer = Keypair::new();
    let (program_config, program_config_bump) = ProgramConfig::find_pda();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/r3_session_keys.so"
    ));
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();
    let client = R3SessionKeysClient::new();

    let ix = client.initialize(payer.pubkey());
    send_tx_expect_ok(&mut svm, ix, &[&payer]);

    let program_config_account = svm.get_account(&program_config).unwrap();
    let mut data: &[u8] = &program_config_account.data;
    let program_config_state = ProgramConfig::try_deserialize(&mut data).unwrap();
    assert_eq!(program_config_state.admin, payer.pubkey());
    assert_eq!(program_config_state.status, ProgramStatus::Active);
    assert_eq!(program_config_state.bump, program_config_bump);
}
