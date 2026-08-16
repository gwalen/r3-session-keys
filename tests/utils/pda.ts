import { PublicKey } from "@solana/web3.js";

// Seeds must stay in sync with the `SEED_PREFIX` constants in programs/r3-session-keys/src/state
export const PROGRAM_CONFIG_SEED = Buffer.from("program_config");
export const USER_SMART_WALLET_SEED = Buffer.from("user_smart_wallet");
export const SESSION_SEED = Buffer.from("session");

export function findProgramConfigPda(programId: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([PROGRAM_CONFIG_SEED], programId);
}

export function deriveProgramConfigPda(programId: PublicKey): PublicKey {
  const [pda] = findProgramConfigPda(programId);
  return pda;
}

export function findUserSmartWalletPda(
  programId: PublicKey,
  smartWalletOwner: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [USER_SMART_WALLET_SEED, smartWalletOwner.toBuffer()],
    programId
  );
}

export function deriveUserSmartWalletPda(
  programId: PublicKey,
  smartWalletOwner: PublicKey
): PublicKey {
  const [pda] = findUserSmartWalletPda(programId, smartWalletOwner);
  return pda;
}

export function findSessionPda(
  programId: PublicKey,
  userSmartWallet: PublicKey,
  sessionKey: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [SESSION_SEED, userSmartWallet.toBuffer(), sessionKey.toBuffer()],
    programId
  );
}

export function deriveSessionPda(
  programId: PublicKey,
  userSmartWallet: PublicKey,
  sessionKey: PublicKey
): PublicKey {
  const [pda] = findSessionPda(programId, userSmartWallet, sessionKey);
  return pda;
}
