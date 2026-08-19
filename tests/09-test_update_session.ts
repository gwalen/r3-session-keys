import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import {
  ComputeBudgetProgram,
  Keypair,
  PublicKey,
  TransactionInstruction,
} from "@solana/web3.js";
import assert from "assert";

import { R3SessionKeys } from "../target/types/r3_session_keys";
import { MockProgram, mockProgramIdl } from "./fixtures/mock-program";
import * as mock from "./fixtures/mock-program";
import {
  DUMMY_ANCHOR_DISCRIMINATOR,
  TARGET_PROGRAM_PLACEHOLDER,
} from "./utils/constants";
import {
  ensureProgramInitialized,
  executeWithSession,
  loadProgramFromFile,
  sendExpectError,
  setAccountData,
  timestampFromFuture,
} from "./utils/helpers";
import * as sessionKeys from "./utils/pda";

describe("09 - Update session", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const baseWallet = provider.wallet as anchor.Wallet;
  const connection = provider.connection;
  const program = anchor.workspace.r3SessionKeys as Program<R3SessionKeys>;

  let mockProgram: Program<MockProgram>;
  let mockCounterPda: PublicKey;

  before(async () => {
    await ensureProgramInitialized(program, baseWallet);
    await loadProgramFromFile(
      connection,
      mock.MOCK_PROGRAM_ID,
      mock.MOCK_PROGRAM_SO_PATH
    );
    mockProgram = new Program(
      mockProgramIdl as MockProgram,
      provider
    ) as Program<MockProgram>;
    mockCounterPda = mock.deriveCounterPda(mockProgram.programId);
  });

  async function initializeMockCounter(authority: PublicKey) {
    const counterAccount = await connection.getAccountInfo(mockCounterPda);
    if (!counterAccount) {
      await mockProgram.methods
        .initialize()
        .accounts({ payer: baseWallet.publicKey })
        .signers([baseWallet.payer])
        .rpc();
    }

    await setAccountData(
      connection,
      mockCounterPda,
      mock.encodeCounterAccount(0, authority)
    );
  }

  async function approveSession(
    smartWalletOwner: Keypair,
    sessionKey: PublicKey,
    makeTransactionUnique = false
  ) {
    const builder = program.methods
      .approveSession(sessionKey)
      .accounts({ smartWalletOwner: smartWalletOwner.publicKey })
      .signers([baseWallet.payer, smartWalletOwner]);

    if (makeTransactionUnique) {
      builder.preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({ units: 200_001 }),
      ]);
    }

    await builder.rpc();
  }

  function incrementInstruction(
    authority: PublicKey
  ): Promise<TransactionInstruction> {
    return mockProgram.methods
      .increment()
      .accounts({ authority })
      .instruction();
  }

  it("resets approval until the owner approves the changed grant", async () => {
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
    await initializeMockCounter(userSmartWallet);

    const expiresAtT1 = await timestampFromFuture(connection);
    const expiresAtT2 = expiresAtT1.add(new anchor.BN(1_000));
    await program.methods
      .createSession(
        sessionKey.publicKey,
        TARGET_PROGRAM_PLACEHOLDER,
        expiresAtT1,
        DUMMY_ANCHOR_DISCRIMINATOR,
        DUMMY_ANCHOR_DISCRIMINATOR.length
      )
      .accounts({
        sessionExecutor: baseWallet.publicKey,
        userSmartWallet,
      })
      .signers([baseWallet.payer])
      .rpc();
    await approveSession(smartWalletOwner, sessionKey.publicKey);

    await program.methods
      .updateSession(
        sessionKey.publicKey,
        mockProgram.programId,
        expiresAtT2,
        mock.INCREMENT_DISCRIMINATOR,
        mock.INCREMENT_DISCRIMINATOR.length
      )
      .accounts({
        sessionExecutor: baseWallet.publicKey,
        userSmartWallet,
      })
      .signers([baseWallet.payer])
      .rpc();

    const updated = await program.account.session.fetch(session);
    assert.equal(
      updated.sessionExecutor.toBase58(),
      baseWallet.publicKey.toBase58()
    );
    assert.equal(
      updated.sessionKey.toBase58(),
      sessionKey.publicKey.toBase58()
    );
    assert.equal(
      updated.targetProgram.toBase58(),
      mockProgram.programId.toBase58()
    );
    assert.ok(updated.expiresAt.eq(expiresAtT2));
    assert.deepEqual(
      Buffer.from(updated.allowedInstructionsDiscriminators),
      mock.INCREMENT_DISCRIMINATOR
    );
    assert.equal(
      updated.discriminatorSize,
      mock.INCREMENT_DISCRIMINATOR.length
    );
    assert.ok("waitingForApproval" in updated.status);

    const incrementIx = await incrementInstruction(userSmartWallet);
    const error = await sendExpectError(
      executeWithSession(program, {
        sessionExecutor: baseWallet.publicKey,
        sessionKey: sessionKey.publicKey,
        userSmartWallet,
        targetInstruction: incrementIx,
      })
        .signers([baseWallet.payer, sessionKey])
        .rpc()
    );
    assert.ok(error.includes("SessionNotApproved"), error);

    await approveSession(smartWalletOwner, sessionKey.publicKey, true);
    await executeWithSession(program, {
      sessionExecutor: baseWallet.publicKey,
      sessionKey: sessionKey.publicKey,
      userSmartWallet,
      targetInstruction: incrementIx,
    })
      .signers([baseWallet.payer, sessionKey])
      .rpc();

    const counter = await mockProgram.account.counter.fetch(mockCounterPda);
    assert.equal(counter.count.toNumber(), 1);
    assert.equal(counter.authority.toBase58(), userSmartWallet.toBase58());
  });
});
