import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { Keypair, PublicKey, TransactionInstruction } from "@solana/web3.js";
import assert from "assert";

import { R3SessionKeys } from "../target/types/r3_session_keys";
import * as sessionKeys from "./utils/pda";
import {
  ensureProgramInitialized,
  executeWithSession,
  futureExpiresAt,
  getOnChainUnixTimestamp,
  loadProgramFromFile,
  sendExpectError,
  setAccountData,
} from "./utils/helpers";
import { ANCHOR_DISCRIMINATOR_LEN, TARGET_PROGRAM_PLACEHOLDER } from "./utils/constants";
import { MockProgram, mockProgramIdl } from "./fixtures/mock-program";
import * as mock from "./fixtures/mock-program";

describe("05 - Execute with session (counter instruction)", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const baseWallet = provider.wallet as anchor.Wallet;
  const connection = provider.connection;

  const program = anchor.workspace.r3SessionKeys as Program<R3SessionKeys>;
  let mockProgram: Program<MockProgram>;

  let mockCounterPda: PublicKey;

  before(async () => {
    await ensureProgramInitialized(program, baseWallet);

    await loadProgramFromFile(connection, mock.MOCK_PROGRAM_ID, mock.MOCK_PROGRAM_SO_PATH);
    mockProgram = new Program(mockProgramIdl as MockProgram, provider) as Program<MockProgram>;
    mockCounterPda = mock.deriveCounterPda(mockProgram.programId);
  });

  /**
   * The mock initialize instruction makes its payer the counter authority.
   * The counter data is overwritten with a cheatcode so the test can exercise PDA signing.
   */
  async function initializeMockCounter(authority: PublicKey) {
    const counterAccount = await connection.getAccountInfo(mockCounterPda);
    if (!counterAccount) {
      await mockProgram.methods
        .initialize()
        .accounts({
          payer: baseWallet.publicKey,
        })
        .signers([baseWallet.payer])
        .rpc()
        .catch((e) => {
          console.log("Error: ", e);
          throw e;
        });
    }

    await setAccountData(connection, mockCounterPda, mock.encodeCounterAccount(0, authority));
  }

  async function createSmartWallet(smartWalletOwner: PublicKey): Promise<PublicKey> {
    const userSmartWalletPda = sessionKeys.deriveUserSmartWalletPda(
      program.programId,
      smartWalletOwner
    );

    await program.methods
      .createSmartWallet(smartWalletOwner)
      .accounts({
        admin: baseWallet.publicKey,
      })
      .signers([baseWallet.payer])
      .rpc()
      .catch((e) => {
        console.log("Error: ", e);
        throw e;
      });

    return userSmartWalletPda;
  }

  async function createSession(params: {
    userSmartWallet: PublicKey;
    sessionKey: PublicKey;
    targetProgram: PublicKey;
    expiresAt: anchor.BN;
    allowedInstructionsDiscriminators: Buffer;
    discriminatorLen: number;
  }) {
    await program.methods
      .createSession(
        params.sessionKey,
        params.targetProgram,
        params.expiresAt,
        params.allowedInstructionsDiscriminators,
        params.discriminatorLen
      )
      .accounts({
        sessionExecutor: baseWallet.publicKey,
        userSmartWallet: params.userSmartWallet,
      })
      .signers([baseWallet.payer])
      .rpc()
      .catch((e) => {
        console.log("Error: ", e);
        throw e;
      });
  }

  async function approveSession(smartWalletOwner: Keypair, sessionKey: PublicKey) {
    await program.methods
      .approveSession(sessionKey)
      .accounts({
        smartWalletOwner: smartWalletOwner.publicKey,
      })
      .signers([baseWallet.payer, smartWalletOwner])
      .rpc()
      .catch((e) => {
        console.log("Error: ", e);
        throw e;
      });
  }

  async function revokeSession(smartWalletOwner: Keypair, sessionKey: PublicKey) {
    await program.methods
      .revokeSession(sessionKey)
      .accounts({
        smartWalletOwner: smartWalletOwner.publicKey,
      })
      .signers([baseWallet.payer, smartWalletOwner])
      .rpc()
      .catch((e) => {
        console.log("Error: ", e);
        throw e;
      });
  }

  function incrementInstruction(authority: PublicKey): Promise<TransactionInstruction> {
    return mockProgram.methods
      .increment()
      .accounts({
        authority,
      })
      .instruction();
  }

  it("Execute mock increment with session", async () => {
    const smartWalletOwner = Keypair.generate();
    const sessionKey = Keypair.generate();
    const userSmartWallet = await createSmartWallet(smartWalletOwner.publicKey);

    await initializeMockCounter(userSmartWallet);
    const incrementIx = await incrementInstruction(userSmartWallet);
    assert.ok(incrementIx.data.subarray(0, 8).equals(mock.INCREMENT_DISCRIMINATOR));

    await createSession({
      userSmartWallet,
      sessionKey: sessionKey.publicKey,
      targetProgram: mockProgram.programId,
      expiresAt: await futureExpiresAt(connection),
      allowedInstructionsDiscriminators: mock.INCREMENT_DISCRIMINATOR,
      discriminatorLen: mock.INCREMENT_DISCRIMINATOR.length,
    });
    await approveSession(smartWalletOwner, sessionKey.publicKey);

    await executeWithSession(program, {
      sessionExecutor: baseWallet.publicKey,
      sessionKey: sessionKey.publicKey,
      userSmartWallet,
      targetInstruction: incrementIx,
    })
      .signers([baseWallet.payer, sessionKey])
      .rpc()
      .catch((e) => {
        console.log("Error: ", e);
        throw e;
      });

    const counter = await mockProgram.account.counter.fetch(mockCounterPda);
    assert.equal(counter.count.toNumber(), 1);
    assert.equal(counter.authority.toBase58(), userSmartWallet.toBase58());
  });

  it("Execute rejects target program not authorized by session", async () => {
    const smartWalletOwner = Keypair.generate();
    const sessionKey = Keypair.generate();
    const userSmartWallet = await createSmartWallet(smartWalletOwner.publicKey);

    await initializeMockCounter(userSmartWallet);
    const incrementIx = await incrementInstruction(userSmartWallet);

    // The session is bound to the system program, the executed instruction targets the mock program
    await createSession({
      userSmartWallet,
      sessionKey: sessionKey.publicKey,
      targetProgram: TARGET_PROGRAM_PLACEHOLDER,
      expiresAt: await futureExpiresAt(connection),
      allowedInstructionsDiscriminators: mock.INCREMENT_DISCRIMINATOR,
      discriminatorLen: mock.INCREMENT_DISCRIMINATOR.length,
    });
    await approveSession(smartWalletOwner, sessionKey.publicKey);

    await sendExpectError(
      executeWithSession(program, {
        sessionExecutor: baseWallet.publicKey,
        sessionKey: sessionKey.publicKey,
        userSmartWallet,
        targetInstruction: incrementIx,
      })
        .signers([baseWallet.payer, sessionKey])
        .rpc()
    );
  });

  it("Execute mock increment not allowed by session", async () => {
    const smartWalletOwner = Keypair.generate();
    const sessionKey = Keypair.generate();
    const userSmartWallet = await createSmartWallet(smartWalletOwner.publicKey);

    await initializeMockCounter(userSmartWallet);
    const incrementIx = await incrementInstruction(userSmartWallet);

    // No discriminator is allowed by this session
    await createSession({
      userSmartWallet,
      sessionKey: sessionKey.publicKey,
      targetProgram: mockProgram.programId,
      expiresAt: await futureExpiresAt(connection),
      allowedInstructionsDiscriminators: Buffer.alloc(0),
      discriminatorLen: ANCHOR_DISCRIMINATOR_LEN,
    });
    await approveSession(smartWalletOwner, sessionKey.publicKey);

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

    assert.ok(error.includes("NotAllowedInstructionDiscriminator"), error);
    const counter = await mockProgram.account.counter.fetch(mockCounterPda);
    assert.equal(counter.count.toNumber(), 0);
    assert.equal(counter.authority.toBase58(), userSmartWallet.toBase58());
  });

  it("Execute mock increment with expired session key", async () => {
    const smartWalletOwner = Keypair.generate();
    const sessionKey = Keypair.generate();
    const userSmartWallet = await createSmartWallet(smartWalletOwner.publicKey);

    await initializeMockCounter(userSmartWallet);
    const incrementIx = await incrementInstruction(userSmartWallet);

    // Expiration is exclusive, so a session expiring at the current timestamp is expired.
    const expiresAt = new anchor.BN(await getOnChainUnixTimestamp(connection));
    await createSession({
      userSmartWallet,
      sessionKey: sessionKey.publicKey,
      targetProgram: mockProgram.programId,
      expiresAt,
      allowedInstructionsDiscriminators: mock.INCREMENT_DISCRIMINATOR,
      discriminatorLen: mock.INCREMENT_DISCRIMINATOR.length,
    });
    await approveSession(smartWalletOwner, sessionKey.publicKey);

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

    assert.ok(error.includes("SessionExpired"), error);
    const counter = await mockProgram.account.counter.fetch(mockCounterPda);
    assert.equal(counter.count.toNumber(), 0);
    assert.equal(counter.authority.toBase58(), userSmartWallet.toBase58());
  });

  it("Execute mock increment with revoked session", async () => {
    const smartWalletOwner = Keypair.generate();
    const sessionKey = Keypair.generate();
    const userSmartWallet = await createSmartWallet(smartWalletOwner.publicKey);

    await initializeMockCounter(userSmartWallet);
    const incrementIx = await incrementInstruction(userSmartWallet);

    await createSession({
      userSmartWallet,
      sessionKey: sessionKey.publicKey,
      targetProgram: mockProgram.programId,
      expiresAt: await futureExpiresAt(connection),
      allowedInstructionsDiscriminators: mock.INCREMENT_DISCRIMINATOR,
      discriminatorLen: mock.INCREMENT_DISCRIMINATOR.length,
    });
    await approveSession(smartWalletOwner, sessionKey.publicKey);
    await revokeSession(smartWalletOwner, sessionKey.publicKey);

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
    const counter = await mockProgram.account.counter.fetch(mockCounterPda);
    assert.equal(counter.count.toNumber(), 0);
    assert.equal(counter.authority.toBase58(), userSmartWallet.toBase58());
  });
});
