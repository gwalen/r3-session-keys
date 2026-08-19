import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { Keypair } from "@solana/web3.js";
import assert from "assert";

import { R3SessionKeys } from "../target/types/r3_session_keys";
import * as sessionKeys from "./utils/pda";
import {
  ensureProgramInitialized,
  sendExpectError,
  timestampFromFuture,
} from "./utils/helpers";
import {
  ANCHOR_DISCRIMINATOR_LEN,
  DUMMY_ANCHOR_DISCRIMINATOR,
  TARGET_PROGRAM_PLACEHOLDER,
} from "./utils/constants";

describe("10 - Close session", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const baseWallet = provider.wallet as anchor.Wallet;
  const connection = provider.connection;
  const program = anchor.workspace.r3SessionKeys as Program<R3SessionKeys>;

  before(async () => {
    await ensureProgramInitialized(program, baseWallet);
  });

  async function createSession() {
    const smartWalletOwner = Keypair.generate();
    const sessionKey = Keypair.generate();
    const userSmartWallet = sessionKeys.deriveUserSmartWalletPda(
      program.programId,
      smartWalletOwner.publicKey
    );
    const session = sessionKeys.deriveSessionPda(
      program.programId,
      userSmartWallet,
      sessionKey.publicKey
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
        await timestampFromFuture(connection),
        DUMMY_ANCHOR_DISCRIMINATOR,
        ANCHOR_DISCRIMINATOR_LEN
      )
      .accounts({
        sessionExecutor: baseWallet.publicKey,
        userSmartWallet,
      })
      .signers([baseWallet.payer])
      .rpc();

    return { sessionKey, userSmartWallet, session };
  }

  it("Close session returns rent to executor", async () => {
    const { sessionKey, userSmartWallet, session } = await createSession();
    const executorBalanceBefore = await connection.getBalance(
      baseWallet.publicKey
    );
    const sessionAccount = await connection.getAccountInfo(session);
    assert.ok(sessionAccount, "session account should exist before close");

    const signature = await program.methods
      .closeSession(sessionKey.publicKey)
      .accounts({
        sessionExecutor: baseWallet.publicKey,
        userSmartWallet,
      })
      .signers([baseWallet.payer])
      .rpc();
    const closeTransaction = await connection.getTransaction(signature, {
      commitment: "confirmed",
      maxSupportedTransactionVersion: 0,
    });
    assert.ok(
      closeTransaction?.meta,
      "close transaction metadata should exist"
    );

    assert.equal(await connection.getAccountInfo(session), null);
    assert.equal(
      await connection.getBalance(baseWallet.publicKey),
      executorBalanceBefore + sessionAccount.lamports - closeTransaction.meta.fee
    );
  });

  it("Close rejects non-executor", async () => {
    const { sessionKey, userSmartWallet, session } = await createSession();
    const wrongExecutor = Keypair.generate();

    const error = await sendExpectError(
      program.methods
        .closeSession(sessionKey.publicKey)
        .accounts({
          sessionExecutor: wrongExecutor.publicKey,
          userSmartWallet,
        })
        .signers([baseWallet.payer, wrongExecutor])
        .rpc()
    );

    assert.ok(error.includes("UnauthorizedSessionExecutor"), error);
    assert.ok(await connection.getAccountInfo(session));
  });
});
