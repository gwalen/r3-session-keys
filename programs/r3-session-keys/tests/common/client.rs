use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{
            instruction::{AccountMeta, Instruction},
            system_program,
        },
        InstructionData, ToAccountMetas,
    },
    r3_session_keys::{
        accounts, instruction,
        state::{
            counter::Counter, program_config::ProgramConfig, session::Session,
            user_smart_wallet::UserSmartWallet,
        },
    },
};

fn build_ix(accounts: Vec<AccountMeta>, data: impl InstructionData) -> Instruction {
    Instruction {
        program_id: r3_session_keys::ID,
        accounts,
        data: data.data(),
    }
}

pub fn initialize(admin: Pubkey) -> Instruction {
    let (program_config, _) = ProgramConfig::find_pda();
    let (counter, _) = Counter::find_pda();
    build_ix(
        accounts::Initialize {
            admin,
            program_config,
            counter,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        instruction::Initialize {},
    )
}

pub fn create_smart_wallet(admin: Pubkey, user_wallet: Pubkey) -> (Instruction, Pubkey, u8) {
    let (program_config, _) = ProgramConfig::find_pda();
    let (user_smart_wallet, bump) = UserSmartWallet::find_pda(&user_wallet);
    let ix = build_ix(
        accounts::CreateSmartWallet {
            admin,
            program_config,
            user_smart_wallet,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        instruction::CreateSmartWallet { user_wallet },
    );
    (ix, user_smart_wallet, bump)
}

pub fn create_session(
    session_executor: Pubkey,
    user_smart_wallet: Pubkey,
    session_key: Pubkey,
    expires_at: i64,
    allowed_instructions_discriminators: Vec<u8>,
    discriminator_len: u8,
) -> (Instruction, Pubkey, u8) {
    let (program_config, _) = ProgramConfig::find_pda();
    let (session, bump) = Session::find_pda(&user_smart_wallet, &session_key);
    let ix = build_ix(
        accounts::CreateSession {
            session_executor,
            program_config,
            user_smart_wallet,
            session,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        instruction::CreateSession {
            session_key,
            expires_at,
            allowed_instructions_discriminators,
            discriminator_len,
        },
    );
    (ix, session, bump)
}

pub fn approve_session(
    smart_wallet_owner: Pubkey,
    user_smart_wallet: Pubkey,
    session_key: Pubkey,
) -> Instruction {
    let (program_config, _) = ProgramConfig::find_pda();
    let (session, _) = Session::find_pda(&user_smart_wallet, &session_key);
    build_ix(
        accounts::ApproveSession {
            smart_wallet_owner,
            program_config,
            user_smart_wallet,
            session,
        }
        .to_account_metas(None), // TODO: ...
        instruction::ApproveSession {
            _session_key: session_key,
        },
    )
}

pub fn revoke_session(
    smart_wallet_owner: Pubkey,
    user_smart_wallet: Pubkey,
    session_key: Pubkey,
) -> Instruction {
    let (program_config, _) = ProgramConfig::find_pda();
    let (session, _) = Session::find_pda(&user_smart_wallet, &session_key);
    build_ix(
        accounts::RevokeSession {
            smart_wallet_owner,
            program_config,
            user_smart_wallet,
            session,
        }
        .to_account_metas(None),
        instruction::RevokeSession {
            _session_key: session_key,
        },
    )
}

pub fn pause(admin: Pubkey) -> Instruction {
    let (program_config, _) = ProgramConfig::find_pda();
    build_ix(
        accounts::Pause {
            admin,
            program_config,
        }
        .to_account_metas(None),
        instruction::Pause {},
    )
}

pub fn unpause(admin: Pubkey) -> Instruction {
    let (program_config, _) = ProgramConfig::find_pda();
    build_ix(
        accounts::Unpause {
            admin,
            program_config,
        }
        .to_account_metas(None),
        instruction::Unpause {},
    )
}
