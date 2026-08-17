import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import assert from "assert";

import { R3SessionKeys } from "../target/types/r3_session_keys";
import * as sessionKeys from "./utils/pda";
import {
  assertProgramActive,
  assertProgramPaused,
  ensureProgramInitialized,
  timestampFromFuture,
  sendExpectError,
} from "./utils/helpers";
import {
  DUMMY_ANCHOR_DISCRIMINATOR,
  ANCHOR_DISCRIMINATOR_LEN,
  TARGET_PROGRAM_PLACEHOLDER,
} from "./utils/constants";

describe("02 - Pause", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const baseWallet = provider.wallet as anchor.Wallet;

  const program = anchor.workspace.r3SessionKeys as Program<R3SessionKeys>;

  before(async () => {
    await ensureProgramInitialized(program, baseWallet);
  });

  function createSmartWallet(userWallet: PublicKey) {
    return program.methods
      .createSmartWallet(userWallet)
      .accounts({
        admin: baseWallet.publicKey,
      })
      .signers([baseWallet.payer]);
  }

  async function createSession(userSmartWallet: PublicKey, sessionKey: PublicKey) {
    return program.methods
      .createSession(
        sessionKey,
        TARGET_PROGRAM_PLACEHOLDER,
        await timestampFromFuture(provider.connection),
        DUMMY_ANCHOR_DISCRIMINATOR,
        ANCHOR_DISCRIMINATOR_LEN
      )
      .accounts({
        sessionExecutor: baseWallet.publicKey,
        userSmartWallet,
      })
      .signers([baseWallet.payer]);
  }

  it("Pause blocks creates and unpause restores", async () => {
    const userWallet = Keypair.generate().publicKey;
    const otherUserWallet = Keypair.generate().publicKey;
    const sessionKey = Keypair.generate().publicKey;

    const userSmartWalletPda = sessionKeys.deriveUserSmartWalletPda(program.programId, userWallet);
    await createSmartWallet(userWallet).rpc();

    await program.methods
      .pause()
      .accounts({
        admin: baseWallet.publicKey,
      })
      .signers([baseWallet.payer])
      .rpc()
      .catch((e) => {
        console.log("Error: ", e);
        throw e;
      });
    await assertProgramPaused(program);

    // program paused so other program functions should fail

    await sendExpectError(createSmartWallet(otherUserWallet).rpc());
    await assertProgramPaused(program);

    await sendExpectError((await createSession(userSmartWalletPda, sessionKey)).rpc());
    await assertProgramPaused(program);

    await program.methods
      .unpause()
      .accounts({
        admin: baseWallet.publicKey,
      })
      .signers([baseWallet.payer])
      .rpc()
      .catch((e) => {
        console.log("Error: ", e);
        throw e;
      });
    await assertProgramActive(program);

    // program unpaused so other program functions should succeed

    await createSmartWallet(otherUserWallet).rpc();
    const otherSmartWallet = await program.account.userSmartWallet.fetch(
      sessionKeys.deriveUserSmartWalletPda(program.programId, otherUserWallet)
    );
    assert.equal(otherSmartWallet.smartWalletOwner.toBase58(), otherUserWallet.toBase58());

    await (await createSession(userSmartWalletPda, sessionKey)).rpc();
    const session = await program.account.session.fetch(
      sessionKeys.deriveSessionPda(program.programId, userSmartWalletPda, sessionKey)
    );
    assert.equal(session.sessionKey.toBase58(), sessionKey.toBase58());
  });
});
