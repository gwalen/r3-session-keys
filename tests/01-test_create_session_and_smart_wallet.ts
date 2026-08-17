import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { Keypair } from "@solana/web3.js";
import assert from "assert";

import { R3SessionKeys } from "../target/types/r3_session_keys";
import * as sessionKeys from "./utils/pda";
import { ensureProgramInitialized, timestampFromFuture } from "./utils/helpers";
import {
  DUMMY_ANCHOR_DISCRIMINATOR,
  ANCHOR_DISCRIMINATOR_LEN,
  TARGET_PROGRAM_PLACEHOLDER,
} from "./utils/constants";

describe("01 - Create smart wallet and session", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const baseWallet = provider.wallet as anchor.Wallet;
  const connection = provider.connection;

  const program = anchor.workspace.r3SessionKeys as Program<R3SessionKeys>;

  before(async () => {
    await ensureProgramInitialized(program, baseWallet);
  });

  it("Create smart wallet", async () => {
    const userWallet = Keypair.generate().publicKey;
    const [userSmartWalletPda, bump] = sessionKeys.findUserSmartWalletPda(
      program.programId,
      userWallet
    );

    await program.methods
      .createSmartWallet(userWallet)
      .accounts({
        admin: baseWallet.publicKey,
      })
      .signers([baseWallet.payer])
      .rpc()
      .catch((e) => {
        console.log("Error: ", e);
        throw e;
      });

    const userSmartWallet = await program.account.userSmartWallet.fetch(userSmartWalletPda);
    assert.equal(userSmartWallet.smartWalletOwner.toBase58(), userWallet.toBase58());
    assert.equal(userSmartWallet.bump, bump);
  });

  it("Create session", async () => {
    const userWallet = Keypair.generate().publicKey;
    const sessionKey = Keypair.generate().publicKey;
    const expiresAt = await timestampFromFuture(connection);

    const userSmartWalletPda = sessionKeys.deriveUserSmartWalletPda(program.programId, userWallet);
    await program.methods
      .createSmartWallet(userWallet)
      .accounts({
        admin: baseWallet.publicKey,
      })
      .signers([baseWallet.payer])
      .rpc()
      .catch((e) => {
        console.log("Error: ", e);
        throw e;
      });

    const [sessionPda, bump] = sessionKeys.findSessionPda(
      program.programId,
      userSmartWalletPda,
      sessionKey
    );

    await program.methods
      .createSession(
        sessionKey,
        TARGET_PROGRAM_PLACEHOLDER,
        expiresAt,
        DUMMY_ANCHOR_DISCRIMINATOR,
        ANCHOR_DISCRIMINATOR_LEN
      )
      .accounts({
        sessionExecutor: baseWallet.publicKey,
        userSmartWallet: userSmartWalletPda,
      })
      .signers([baseWallet.payer])
      .rpc()
      .catch((e) => {
        console.log("Error: ", e);
        throw e;
      });

    const session = await program.account.session.fetch(sessionPda);
    assert.equal(session.sessionExecutor.toBase58(), baseWallet.publicKey.toBase58());
    assert.equal(session.sessionKey.toBase58(), sessionKey.toBase58());
    assert.equal(session.targetProgram.toBase58(), TARGET_PROGRAM_PLACEHOLDER.toBase58());
    assert.equal(session.expiresAt.toString(), expiresAt.toString());
    assert.equal(
      Buffer.from(session.allowedInstructionsDiscriminators).toString("hex"),
      DUMMY_ANCHOR_DISCRIMINATOR.toString("hex")
    );
    assert.equal(session.discriminatorSize, ANCHOR_DISCRIMINATOR_LEN);
    assert.ok("waitingForApproval" in session.status);
    assert.equal(session.bump, bump);
  });
});
