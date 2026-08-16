import { PublicKey } from "@solana/web3.js";
import {
  getAssociatedTokenAddressSync,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import * as path from "path";

export * from "./types/mock_program";

// The IDL is the one consumed by `declare_program!(mock_program)` in the Rust tests
import mockProgramIdl from "../../../idls/mock_program.json";
export { mockProgramIdl };

export const MOCK_PROGRAM_ID = new PublicKey(mockProgramIdl.address);
export const MOCK_PROGRAM_SO_PATH = path.join(__dirname, "mock_program.so");

// Seeds must stay in sync with mock_program::constants
export const COUNTER_SEED = Buffer.from("counter");
export const POOL_SEED = Buffer.from("pool");
export const LP_MINT_SEED = Buffer.from("lp_mint");

export function deriveCounterPda(programId: PublicKey): PublicKey {
  const [pda] = PublicKey.findProgramAddressSync([COUNTER_SEED], programId);
  return pda;
}

export function derivePoolPda(programId: PublicKey, depositMint: PublicKey): PublicKey {
  const [pda] = PublicKey.findProgramAddressSync(
    [POOL_SEED, depositMint.toBuffer()],
    programId
  );
  return pda;
}

export function deriveLpMintPda(programId: PublicKey, pool: PublicKey): PublicKey {
  const [pda] = PublicKey.findProgramAddressSync([LP_MINT_SEED, pool.toBuffer()], programId);
  return pda;
}

// The pool vault is the pool PDA's associated token account for the deposit mint
export function deriveVaultAddress(pool: PublicKey, depositMint: PublicKey): PublicKey {
  return getAssociatedTokenAddressSync(depositMint, pool, true, TOKEN_PROGRAM_ID);
}

export function deriveUserDepositAccount(user: PublicKey, depositMint: PublicKey): PublicKey {
  return getAssociatedTokenAddressSync(depositMint, user, true, TOKEN_PROGRAM_ID);
}

// LP mints are Token-2022 mints created by the mock program
export function deriveUserLpAccount(user: PublicKey, lpMint: PublicKey): PublicKey {
  return getAssociatedTokenAddressSync(lpMint, user, true, TOKEN_2022_PROGRAM_ID);
}

/*** Discriminators (the TS counterpart of `args::X::DISCRIMINATOR` in the Rust mock client) ***/

export function instructionDiscriminator(instructionName: string): Buffer {
  const instruction = mockProgramIdl.instructions.find((ix) => ix.name === instructionName);
  if (!instruction) {
    throw new Error(`Instruction ${instructionName} not found in the mock program IDL`);
  }
  return Buffer.from(instruction.discriminator);
}

export function accountDiscriminator(accountName: string): Buffer {
  const account = mockProgramIdl.accounts.find((acc) => acc.name === accountName);
  if (!account) {
    throw new Error(`Account ${accountName} not found in the mock program IDL`);
  }
  return Buffer.from(account.discriminator);
}

export const INCREMENT_DISCRIMINATOR = instructionDiscriminator("increment");
export const DEPOSIT_DISCRIMINATOR = instructionDiscriminator("deposit");
export const WITHDRAW_DISCRIMINATOR = instructionDiscriminator("withdraw");

/**
 * Serializes a `Counter` account so it can be written with the surfnet_setAccount cheatcode.
 * Layout: [discriminator: 8][count: u64 LE][authority: pubkey]
 * Used to zero the counter accounts between some tests
 */
export function encodeCounterAccount(count: number, authority: PublicKey): Buffer {
  const data = Buffer.alloc(8 + 8 + 32);
  accountDiscriminator("Counter").copy(data, 0);
  data.writeBigUInt64LE(BigInt(count), 8);
  authority.toBuffer().copy(data, 16);
  return data;
}
