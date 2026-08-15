use {
    super::{client, send_tx},
    anchor_lang::prelude::Pubkey,
    litesvm::LiteSVM,
    r3_session_keys::state::program_config::ProgramConfig,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

pub struct Env {
    pub svm: LiteSVM,
    pub program_id: Pubkey,
    pub admin: Keypair,
    pub program_config: Pubkey,
}

impl Env {
    pub fn new() -> Self {
        let program_id = r3_session_keys::id();
        let admin = Keypair::new();
        let (program_config, _) = ProgramConfig::find_pda();

        let mut svm = LiteSVM::new();
        let bytes = include_bytes!(concat!(
            env!("CARGO_TARGET_TMPDIR"),
            "/../deploy/r3_session_keys.so"
        ));
        svm.add_program(program_id, bytes).unwrap();
        svm.airdrop(&admin.pubkey(), 1_000_000_000).unwrap();
        send_tx(&mut svm, client::initialize(admin.pubkey()), &[&admin]);

        Self {
            svm,
            program_id,
            admin,
            program_config,
        }
    }
}
