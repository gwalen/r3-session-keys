import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { Keypair, TransactionInstruction } from "@solana/web3.js";
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

describe("07 - Negative authorization", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const baseWallet = provider.wallet as anchor.Wallet;
  const program = anchor.workspace.r3SessionKeys as Program<R3SessionKeys>;

  before(async () => {
    await ensureProgramInitialized(program, baseWallet);
  });

  it("Rejects approve by a non-owner", async () => {
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

    const session = sessionKeys.deriveSessionPda(
      program.programId,
      userSmartWallet,
      sessionKey.publicKey
    );
    const wrongOwner = Keypair.generate();
    const error = await sendExpectError(
      program.methods
        .approveSession(sessionKey.publicKey)
        // Override auto-resolved PDAs to deliberately pair the wrong signer with this wallet.
        .accountsStrict({
          smartWalletOwner: wrongOwner.publicKey,
          programConfig: sessionKeys.deriveProgramConfigPda(program.programId),
          userSmartWallet,
          session,
        })
        .signers([baseWallet.payer, wrongOwner])
        .rpc()
    );

    assert.ok(error.includes("ConstraintSeeds"), error);
    const state = await program.account.session.fetch(session);
    assert.ok("waitingForApproval" in state.status);
  });

  it("Rejects revoke by a non-owner", async () => {
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

    const session = sessionKeys.deriveSessionPda(
      program.programId,
      userSmartWallet,
      sessionKey.publicKey
    );
    const wrongOwner = Keypair.generate();
    const error = await sendExpectError(
      program.methods
        .revokeSession(sessionKey.publicKey)
        // Override auto-resolved PDAs to deliberately pair the wrong signer with this wallet.
        .accountsStrict({
          smartWalletOwner: wrongOwner.publicKey,
          programConfig: sessionKeys.deriveProgramConfigPda(program.programId),
          userSmartWallet,
          session,
        })
        .signers([baseWallet.payer, wrongOwner])
        .rpc()
    );

    assert.ok(error.includes("ConstraintSeeds"), error);
    const state = await program.account.session.fetch(session);
    assert.ok("waitingForApproval" in state.status);
  });

  it("Rejects an executor the session was not created for", async () => {
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
      .accounts({ smartWalletOwner: smartWalletOwner.publicKey })
      .signers([baseWallet.payer, smartWalletOwner])
      .rpc();

    const wrongExecutor = Keypair.generate();
    const targetInstruction = new TransactionInstruction({
      programId: TARGET_PROGRAM_PLACEHOLDER,
      keys: [{ pubkey: userSmartWallet, isSigner: false, isWritable: false }],
      data: DUMMY_ANCHOR_DISCRIMINATOR,
    });
    const error = await sendExpectError(
      executeWithSession(program, {
        sessionExecutor: wrongExecutor.publicKey,
        sessionKey: sessionKey.publicKey,
        userSmartWallet,
        targetInstruction,
      })
        .signers([baseWallet.payer, wrongExecutor, sessionKey])
        .rpc()
    );

    assert.ok(error.includes("UnauthorizedSessionExecutor"), error);
  });

  it("Rejects pause by a non-admin", async () => {
    const wrongAdmin = Keypair.generate();
    const error = await sendExpectError(
      program.methods
        .pause()
        .accounts({ admin: wrongAdmin.publicKey })
        .signers([baseWallet.payer, wrongAdmin])
        .rpc()
    );

    assert.ok(error.includes("UnauthorizedAdmin"), error);
    const config = await program.account.programConfig.fetch(
      sessionKeys.deriveProgramConfigPda(program.programId)
    );
    assert.ok("active" in config.status);
  });

  it("Rejects unpause by a non-admin", async () => {
    await program.methods
      .pause()
      .accounts({ admin: baseWallet.publicKey })
      .signers([baseWallet.payer])
      .rpc();

    try {
      const wrongAdmin = Keypair.generate();
      const error = await sendExpectError(
        program.methods
          .unpause()
          .accounts({ admin: wrongAdmin.publicKey })
          .signers([baseWallet.payer, wrongAdmin])
          .rpc()
      );

      assert.ok(error.includes("UnauthorizedAdmin"), error);
      const config = await program.account.programConfig.fetch(
        sessionKeys.deriveProgramConfigPda(program.programId)
      );
      assert.ok("paused" in config.status);
    } finally {
      await program.methods
        .unpause()
        .accounts({ admin: baseWallet.publicKey })
        .signers([baseWallet.payer])
        .rpc();
    }
  });
});
