import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { AccountMeta, Keypair, TransactionInstruction } from "@solana/web3.js";
import assert from "assert";

import { R3SessionKeys } from "../target/types/r3_session_keys";
import * as sessionKeys from "./utils/pda";
import {
  ensureProgramInitialized,
  executeWithSession,
  sendExpectError,
  timestampFromFuture,
} from "./utils/helpers";
import {
  ANCHOR_DISCRIMINATOR_LEN,
  DUMMY_ANCHOR_DISCRIMINATOR,
  TARGET_PROGRAM_PLACEHOLDER,
} from "./utils/constants";

describe("08 - Execute validation checks", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const baseWallet = provider.wallet as anchor.Wallet;
  const program = anchor.workspace.r3SessionKeys as Program<R3SessionKeys>;

  before(async () => {
    await ensureProgramInitialized(program, baseWallet);
  });

  async function createApprovedSession() {
    const smartWalletOwner = Keypair.generate();
    const sessionKey = Keypair.generate();
    const userSmartWallet = sessionKeys.deriveUserSmartWalletPda(
      program.programId,
      smartWalletOwner.publicKey
    );

    await program.methods
      .createSmartWallet(smartWalletOwner.publicKey)
      .accounts({ admin: baseWallet.publicKey })
      .signers([baseWallet.payer])
      .rpc();

    await program.methods
      .createSession(
        sessionKey.publicKey,
        TARGET_PROGRAM_PLACEHOLDER,
        await timestampFromFuture(provider.connection),
        DUMMY_ANCHOR_DISCRIMINATOR,
        ANCHOR_DISCRIMINATOR_LEN
      )
      .accounts({
        sessionExecutor: baseWallet.publicKey,
        userSmartWallet,
      })
      .signers([baseWallet.payer])
      .rpc();

    await program.methods
      .approveSession(sessionKey.publicKey)
      .accounts({
        smartWalletOwner: smartWalletOwner.publicKey,
      })
      .signers([baseWallet.payer, smartWalletOwner])
      .rpc();

    return {
      sessionKey,
      userSmartWallet,
      session: sessionKeys.deriveSessionPda(
        program.programId,
        userSmartWallet,
        sessionKey.publicKey
      ),
    };
  }

  async function executeExpectingError(
    fixture: Awaited<ReturnType<typeof createApprovedSession>>,
    keys: AccountMeta[]
  ) {
    const targetInstruction = new TransactionInstruction({
      programId: TARGET_PROGRAM_PLACEHOLDER,
      keys,
      data: DUMMY_ANCHOR_DISCRIMINATOR,
    });

    return sendExpectError(
      executeWithSession(program, {
        sessionExecutor: baseWallet.publicKey,
        sessionKey: fixture.sessionKey.publicKey,
        userSmartWallet: fixture.userSmartWallet,
        targetInstruction,
      })
        .signers([baseWallet.payer, fixture.sessionKey])
        .rpc()
    );
  }

  it("Rejects the session key in remaining accounts", async () => {
    const fixture = await createApprovedSession();

    const error = await executeExpectingError(fixture, [
      {
        pubkey: fixture.sessionKey.publicKey,
        isSigner: false,
        isWritable: false,
      },
      { pubkey: fixture.userSmartWallet, isSigner: false, isWritable: false },
    ]);

    assert.ok(error.includes("RemainingAccountsContainsSessionKey"), error);
  });

  it("Rejects another program-owned account", async () => {
    const fixture = await createApprovedSession();

    const error = await executeExpectingError(fixture, [
      { pubkey: fixture.session, isSigner: false, isWritable: false },
      { pubkey: fixture.userSmartWallet, isSigner: false, isWritable: false },
    ]);

    assert.ok(
      error.includes("RemainingAccountsContainsProgramOwnedAccount"),
      error
    );
  });

  it("Rejects the smart wallet as writable", async () => {
    const fixture = await createApprovedSession();

    const error = await executeExpectingError(fixture, [
      { pubkey: fixture.userSmartWallet, isSigner: false, isWritable: true },
    ]);

    assert.ok(error.includes("UserSmartWalletAccountIsWritable"), error);
  });

  it("Rejects a missing smart wallet", async () => {
    const fixture = await createApprovedSession();

    const error = await executeExpectingError(fixture, []);

    assert.ok(error.includes("UserSmartWalletNotFound"), error);
  });

  it("Rejects a duplicate smart wallet", async () => {
    const fixture = await createApprovedSession();

    const error = await executeExpectingError(fixture, [
      { pubkey: fixture.userSmartWallet, isSigner: false, isWritable: false },
      { pubkey: fixture.userSmartWallet, isSigner: false, isWritable: false },
    ]);

    assert.ok(error.includes("MultipleUserSmartWalletAccounts"), error);
  });
});
