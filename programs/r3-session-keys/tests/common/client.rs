use {
    anchor_client::{Client, Cluster, Program},
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
    },
    r3_session_keys::{
        accounts, instruction,
        state::{
            program_config::ProgramConfig, session::Session, user_smart_wallet::UserSmartWallet,
        },
    },
    solana_keypair::Keypair,
    std::rc::Rc,
};

pub struct R3SessionKeysClient {
    program: Program<Rc<Keypair>>,
}

impl R3SessionKeysClient {
    pub fn new() -> Self {
        let client = Client::new(Cluster::Localnet, Rc::new(Keypair::new()));
        let program = client.program(r3_session_keys::ID).unwrap();
        Self { program }
    }

    pub fn initialize(&self, admin: Pubkey) -> Instruction {
        let (program_config, _) = ProgramConfig::find_pda();
        self.program
            .request()
            .accounts(accounts::Initialize {
                admin,
                program_config,
                system_program: system_program::ID,
            })
            .args(instruction::Initialize {})
            .instructions()
            .remove(0)
    }

    pub fn create_smart_wallet(
        &self,
        admin: Pubkey,
        user_wallet: Pubkey,
    ) -> (Instruction, Pubkey, u8) {
        let (program_config, _) = ProgramConfig::find_pda();
        let (user_smart_wallet, bump) = UserSmartWallet::find_pda(&user_wallet);
        let ix = self
            .program
            .request()
            .accounts(accounts::CreateSmartWallet {
                admin,
                program_config,
                user_smart_wallet,
                system_program: system_program::ID,
            })
            .args(instruction::CreateSmartWallet { user_wallet })
            .instructions()
            .remove(0);
        (ix, user_smart_wallet, bump)
    }

    pub fn create_session(
        &self,
        session_executor: Pubkey,
        user_smart_wallet: Pubkey,
        session_key: Pubkey,
        target_program: Pubkey,
        expires_at: i64,
        allowed_instructions_discriminators: Vec<u8>,
        discriminator_len: u8,
    ) -> (Instruction, Pubkey, u8) {
        let (program_config, _) = ProgramConfig::find_pda();
        let (session, bump) = Session::find_pda(&user_smart_wallet, &session_key);
        let ix = self
            .program
            .request()
            .accounts(accounts::CreateSession {
                session_executor,
                program_config,
                user_smart_wallet,
                session,
                system_program: system_program::ID,
            })
            .args(instruction::CreateSession {
                session_key,
                target_program,
                expires_at,
                allowed_instructions_discriminators,
                discriminator_len,
            })
            .instructions()
            .remove(0);
        (ix, session, bump)
    }

    pub fn approve_session(
        &self,
        smart_wallet_owner: Pubkey,
        user_smart_wallet: Pubkey,
        session_key: Pubkey,
    ) -> Instruction {
        let (program_config, _) = ProgramConfig::find_pda();
        let (session, _) = Session::find_pda(&user_smart_wallet, &session_key);
        self.program
            .request()
            .accounts(accounts::ApproveSession {
                smart_wallet_owner,
                program_config,
                user_smart_wallet,
                session,
            })
            .args(instruction::ApproveSession {
                _session_key: session_key,
            })
            .instructions()
            .remove(0)
    }

    pub fn revoke_session(
        &self,
        smart_wallet_owner: Pubkey,
        user_smart_wallet: Pubkey,
        session_key: Pubkey,
    ) -> Instruction {
        let (program_config, _) = ProgramConfig::find_pda();
        let (session, _) = Session::find_pda(&user_smart_wallet, &session_key);
        self.program
            .request()
            .accounts(accounts::RevokeSession {
                smart_wallet_owner,
                program_config,
                user_smart_wallet,
                session,
            })
            .args(instruction::RevokeSession {
                _session_key: session_key,
            })
            .instructions()
            .remove(0)
    }

    pub fn execute_with_session(
        &self,
        session_executor: Pubkey,
        session_key: Pubkey,
        user_smart_wallet: Pubkey,
        mut target_instruction: Instruction,
    ) -> Instruction {
        let (program_config, _) = ProgramConfig::find_pda();
        let (session, _) = Session::find_pda(&user_smart_wallet, &session_key);
        let target_program = target_instruction.program_id;

        // The smart-wallet PDA cannot sign the outer transaction. The session program
        // promotes it to a signer only for the target CPI via invoke_signed.
        for account in &mut target_instruction.accounts {
            if account.pubkey == user_smart_wallet {
                account.is_signer = false;
            }
        }

        let remaining_accounts = target_instruction.accounts;
        let instruction_data = target_instruction.data;
        self.program
            .request()
            .accounts(accounts::ExecuteWithSession {
                session_executor,
                session_key,
                program_config,
                session,
                user_smart_wallet,
                target_program,
            })
            // Add target instruction accounts as remaining accounts.
            .accounts(remaining_accounts)
            .args(instruction::ExecuteWithSession { instruction_data })
            .instructions()
            .remove(0)
    }

    pub fn pause(&self, admin: Pubkey) -> Instruction {
        let (program_config, _) = ProgramConfig::find_pda();
        self.program
            .request()
            .accounts(accounts::Pause {
                admin,
                program_config,
            })
            .args(instruction::Pause {})
            .instructions()
            .remove(0)
    }

    pub fn unpause(&self, admin: Pubkey) -> Instruction {
        let (program_config, _) = ProgramConfig::find_pda();
        self.program
            .request()
            .accounts(accounts::Unpause {
                admin,
                program_config,
            })
            .args(instruction::Unpause {})
            .instructions()
            .remove(0)
    }
}
