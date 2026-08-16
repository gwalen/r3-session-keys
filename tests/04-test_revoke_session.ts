import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { Keypair } from "@solana/web3.js";
import assert from "assert";

import { R3SessionKeys } from "../target/types/r3_session_keys";
import * as sessionKeys from "./utils/pda";
import { ensureProgramInitialized, futureExpiresAt } from "./utils/helpers";
import { ANCHOR_DISCRIMINATOR_LEN, TARGET_PROGRAM_PLACEHOLDER } from "./utils/constants";

describe("04 - Revoke session", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const baseWallet = provider.wallet as anchor.Wallet;
  const connection = provider.connection;

  const program = anchor.workspace.r3SessionKeys as Program<R3SessionKeys>;

  before(async () => {
    await ensureProgramInitialized(program, baseWallet);
  });

  it("Revoke session changes status", async () => {
    const user = Keypair.generate();
    const sessionKey = Keypair.generate().publicKey;
    const expiresAt = await futureExpiresAt(connection);

    const userSmartWalletPda = sessionKeys.deriveUserSmartWalletPda(
      program.programId,
      user.publicKey
    );
    await program.methods
      .createSmartWallet(user.publicKey)
      .accounts({
        admin: baseWallet.publicKey,
      })
      .signers([baseWallet.payer])
      .rpc()
      .catch((e) => {
        console.log("Error: ", e);
        throw e;
      });

    const sessionPda = sessionKeys.deriveSessionPda(
      program.programId,
      userSmartWalletPda,
      sessionKey
    );
    await program.methods
      .createSession(
        sessionKey,
        TARGET_PROGRAM_PLACEHOLDER,
        expiresAt,
        Buffer.alloc(0),
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

    let session = await program.account.session.fetch(sessionPda);
    assert.ok("waitingForApproval" in session.status);

    await program.methods
      .revokeSession(sessionKey)
      .accounts({
        smartWalletOwner: user.publicKey,
      })
      .signers([baseWallet.payer, user])
      .rpc()
      .catch((e) => {
        console.log("Error: ", e);
        throw e;
      });

    session = await program.account.session.fetch(sessionPda);
    assert.ok("revoked" in session.status);
  });
});
