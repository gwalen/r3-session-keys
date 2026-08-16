import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import {
  AccountMeta,
  Connection,
  PublicKey,
  SYSVAR_CLOCK_PUBKEY,
  TransactionInstruction,
} from "@solana/web3.js";
import assert from "assert";
import * as fs from "fs";

import { R3SessionKeys } from "../../target/types/r3_session_keys";
import * as sessionKeys from "./pda";
import { AIRDROP_SOL_AMOUNT, SESSION_EXPIRY_OFFSET_SECONDS } from "./constants";

export async function airdrop(connection: Connection, userPubkey: PublicKey) {
  const signature = await connection.requestAirdrop(userPubkey, AIRDROP_SOL_AMOUNT);
  const latestBlockHash = await connection.getLatestBlockhash();

  await connection.confirmTransaction({
    blockhash: latestBlockHash.blockhash,
    lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
    signature: signature,
  });
}

/*** Surfpool cheatcodes ***/

/**
 * The LiteSVM tests build a fresh SVM per test (`Env::new()`), load programs and overwrite
 * account data directly. Against a surfnet the equivalent primitives are the `surfnet_*`
 * cheatcodes. Every byte payload they take is hex encoded.
 */
async function surfnetRpc(connection: Connection, method: string, params: unknown[]): Promise<any> {
  const response = await fetch(connection.rpcEndpoint, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params }),
  });

  const body = (await response.json()) as { result?: any; error?: unknown };
  if (body.error) {
    throw new Error(`${method} failed: ${JSON.stringify(body.error)}`);
  }
  return body.result;
}

// The counterpart of `svm.add_program(...)` in the Rust tests
export async function loadProgramFromFile(
  connection: Connection,
  programId: PublicKey,
  soPath: string
) {
  const programAccount = await connection.getAccountInfo(programId);
  if (programAccount) {
    return;
  }

  const bytes = fs.readFileSync(soPath);
  await surfnetRpc(connection, "surfnet_writeProgram", [
    programId.toBase58(),
    bytes.toString("hex"),
    0,
  ]);
}

// The counterpart of `svm.set_account(...)` in the Rust tests
export async function setAccountData(connection: Connection, address: PublicKey, data: Buffer) {
  const account = await connection.getAccountInfo(address);
  assert.ok(account, `Account ${address.toBase58()} must exist before overwriting its data`);

  await surfnetRpc(connection, "surfnet_setAccount", [
    address.toBase58(),
    {
      lamports: account.lamports,
      owner: account.owner.toBase58(),
      data: data.toString("hex"),
    },
  ]);
}

/*** Clock helpers ***/

export async function getOnChainUnixTimestamp(connection: Connection): Promise<number> {
  const clock = await connection.getAccountInfo(SYSVAR_CLOCK_PUBKEY);
  assert.ok(clock, "Clock sysvar should exist");
  // Clock layout: [slot: u64][epoch_start_timestamp: i64][epoch: u64][leader_schedule_epoch: u64][unix_timestamp: i64]
  return Number(clock.data.readBigInt64LE(32));
}

export async function futureExpiresAt(
  connection: Connection,
  offsetSeconds: number = SESSION_EXPIRY_OFFSET_SECONDS
): Promise<anchor.BN> {
  const now = await getOnChainUnixTimestamp(connection);
  return new anchor.BN(now + offsetSeconds);
}

/*** Program helpers ***/

/**
 * The Rust tests initialize a brand new program per test. The surfnet keeps its state for the
 * whole mocha run, so the program config is created once and reused by every test file.
 */
export async function ensureProgramInitialized(
  program: Program<R3SessionKeys>,
  admin: anchor.Wallet
): Promise<PublicKey> {
  const programConfigPda = sessionKeys.deriveProgramConfigPda(program.programId);

  const programConfigAccount = await program.provider.connection.getAccountInfo(programConfigPda);
  if (programConfigAccount) {
    return programConfigPda;
  }

  await program.methods
    .initialize()
    .accounts({
      admin: admin.publicKey,
    })
    .signers([admin.payer])
    .rpc()
    .catch((e) => {
      console.log("Error: ", e);
      throw e;
    });

  return programConfigPda;
}

/**
 * Builds an `execute_with_session` call wrapping a target program instruction.
 * The smart-wallet PDA cannot sign the outer transaction. The session program
 * promotes it to a signer only for the target CPI via invoke_signed.
 */
export function executeWithSession(
  program: Program<R3SessionKeys>,
  params: {
    sessionExecutor: PublicKey;
    sessionKey: PublicKey;
    userSmartWallet: PublicKey;
    targetInstruction: TransactionInstruction;
  }
) {
  const { sessionExecutor, sessionKey, userSmartWallet, targetInstruction } = params;

  const remainingAccounts: AccountMeta[] = targetInstruction.keys.map((account) =>
    account.pubkey.equals(userSmartWallet) ? { ...account, isSigner: false } : account
  );

  return (
    program.methods
      .executeWithSession(targetInstruction.data)
      // user_smart_wallet seeds reference its own `smart_wallet_owner` field, so it is the one
      // account the IDL resolver cannot derive - program_config and session are resolved from it
      .accountsPartial({
        sessionExecutor,
        sessionKey,
        userSmartWallet,
        targetProgram: targetInstruction.programId,
      })
      // Add target instruction accounts as remaining accounts.
      .remainingAccounts(remainingAccounts)
  );
}

/*** Assertion helpers ***/

/**
 * The counterpart of `send_tx_expect_error(...)`: fails the test when the transaction lands and
 * returns the error together with the program logs so the caller can assert on the error code.
 */
export async function sendExpectError(pendingSignature: Promise<string>): Promise<string> {
  try {
    await pendingSignature;
  } catch (e: any) {
    const logs: string[] = e.logs ?? e.transactionLogs ?? [];
    const error = `${e}\n${logs.join("\n")}`;
    // The logs are noisy enough to break up the mocha reporter, so only print them
    // when debugging: `TEST_LOGS=1 just ts`.
    if (process.env.TEST_LOGS) {
      console.log("Transaction failed as expected: ", error);
    }
    return error;
  }

  assert.fail("expected transaction to fail");
}

export async function assertProgramPaused(program: Program<R3SessionKeys>) {
  const programConfig = await program.account.programConfig.fetch(
    sessionKeys.deriveProgramConfigPda(program.programId)
  );
  assert.ok("paused" in programConfig.status, "Program should be paused");
}

export async function assertProgramActive(program: Program<R3SessionKeys>) {
  const programConfig = await program.account.programConfig.fetch(
    sessionKeys.deriveProgramConfigPda(program.programId)
  );
  assert.ok("active" in programConfig.status, "Program should be active");
}
