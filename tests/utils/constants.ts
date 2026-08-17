import { LAMPORTS_PER_SOL, SystemProgram } from "@solana/web3.js";

export const AIRDROP_SOL_AMOUNT = 100 * LAMPORTS_PER_SOL;

// Sessions in the tests are created with a comfortable margin over the on-chain clock
export const SESSION_EXPIRY_OFFSET_SECONDS = 1000;

// AnchorV1 instruction discriminators are 8 bytes
export const ANCHOR_DISCRIMINATOR_LEN = 8;

// Sessions that never execute a CPI point at the system program (mirrors TARGET_PROGRAM_PLACEHOLDER in the Rust tests)
export const TARGET_PROGRAM_PLACEHOLDER = SystemProgram.programId;

export const DUMMY_ANCHOR_DISCRIMINATOR = Buffer.from([
  0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
]);

/*** 06-advanced-tests token pool constants ***/

export const TOKEN_DECIMALS = 6;
export const INITIAL_TOKEN_BALANCE = 1_000_000;
export const DEPOSIT_AMOUNT = 250_000;
