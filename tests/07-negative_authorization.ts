import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { Keypair, PublicKey, TransactionInstruction } from "@solana/web3.js";
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

  async function createSessionFixture() {
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

    return {
      smartWalletOwner,
      sessionKey,
      userSmartWallet,
      session: sessionKeys.deriveSessionPda(
        program.programId,
        userSmartWallet,
        sessionKey.publicKey
      ),
    };
  }

  async function approveSession(signer: Keypair, sessionKey: PublicKey) {
    return program.methods
      .approveSession(sessionKey)
      .accounts({
        smartWalletOwner: signer.publicKey,
      })
      .signers([baseWallet.payer, signer])
      .rpc();
  }

  it("Rejects approve by a non-owner", async () => {
    const fixture = await createSessionFixture();
    const wrongOwner = Keypair.generate();

    const error = await sendExpectError(
      program.methods
        .approveSession(fixture.sessionKey.publicKey)
        // use .accountsStrict() to override an otherwise auto-resolved PDAs
        .accountsStrict({
          smartWalletOwner: wrongOwner.publicKey,
          programConfig: sessionKeys.deriveProgramConfigPda(program.programId),
          userSmartWallet: fixture.userSmartWallet,
          session: fixture.session,
        })
        .signers([baseWallet.payer, wrongOwner])
        .rpc()
    );

    assert.ok(error.includes("ConstraintSeeds"), error);
    const session = await program.account.session.fetch(fixture.session);
    assert.ok("waitingForApproval" in session.status);
  });

  it("Rejects revoke by a non-owner", async () => {
    const fixture = await createSessionFixture();
    const wrongOwner = Keypair.generate();

    const error = await sendExpectError(
      program.methods
        .revokeSession(fixture.sessionKey.publicKey)
        // use .accountsStrict() to override an otherwise auto-resolved PDAs
        .accountsStrict({
          smartWalletOwner: wrongOwner.publicKey,
          programConfig: sessionKeys.deriveProgramConfigPda(program.programId),
          userSmartWallet: fixture.userSmartWallet,
          session: fixture.session,
        })
        .signers([baseWallet.payer, wrongOwner])
        .rpc()
    );

    assert.ok(error.includes("ConstraintSeeds"), error);
    const session = await program.account.session.fetch(fixture.session);
    assert.ok("waitingForApproval" in session.status);
  });

  it("Rejects an executor the session was not created for", async () => {
    const fixture = await createSessionFixture();
    await approveSession(
      fixture.smartWalletOwner,
      fixture.sessionKey.publicKey
    );

    const wrongExecutor = Keypair.generate();
    const targetInstruction = new TransactionInstruction({
      programId: TARGET_PROGRAM_PLACEHOLDER,
      keys: [
        { pubkey: fixture.userSmartWallet, isSigner: false, isWritable: false },
      ],
      data: DUMMY_ANCHOR_DISCRIMINATOR,
    });
    const error = await sendExpectError(
      executeWithSession(program, {
        sessionExecutor: wrongExecutor.publicKey,
        sessionKey: fixture.sessionKey.publicKey,
        userSmartWallet: fixture.userSmartWallet,
        targetInstruction,
      })
        .signers([baseWallet.payer, wrongExecutor, fixture.sessionKey])
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
