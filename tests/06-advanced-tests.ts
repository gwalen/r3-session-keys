import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  createAssociatedTokenAccount,
  createMint,
  getAccount,
  getMint,
  mintTo,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import assert from "assert";

import { R3SessionKeys } from "../target/types/r3_session_keys";
import * as sessionKeys from "./utils/pda";
import {
  ensureProgramInitialized,
  executeWithSession,
  timestampFromFuture,
  loadProgramFromFile,
  sendExpectError,
} from "./utils/helpers";
import { DEPOSIT_AMOUNT, INITIAL_TOKEN_BALANCE, TOKEN_DECIMALS } from "./utils/constants";
import { MockProgram, mockProgramIdl } from "./fixtures/mock-program";
import * as mock from "./fixtures/mock-program";

interface TokenPoolFixture {
  depositMint: PublicKey;
  lpMint: PublicKey;
  vault: PublicKey;
  userDepositAccount: PublicKey;
  userLpAccount: PublicKey;
}

describe("06 - Advanced tests (token pool)", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const baseWallet = provider.wallet as anchor.Wallet;
  const connection = provider.connection;

  const program = anchor.workspace.r3SessionKeys as Program<R3SessionKeys>;
  let mockProgram: Program<MockProgram>;

  before(async () => {
    await ensureProgramInitialized(program, baseWallet);

    await loadProgramFromFile(connection, mock.MOCK_PROGRAM_ID, mock.MOCK_PROGRAM_SO_PATH);
    mockProgram = new Program(mockProgramIdl as MockProgram, provider) as Program<MockProgram>;
  });

  async function initializeMockTokenPool(tokenOwner: PublicKey): Promise<TokenPoolFixture> {
    const depositMint = await createMint(
      connection,
      baseWallet.payer,
      baseWallet.publicKey,
      null,
      TOKEN_DECIMALS,
      undefined,
      undefined,
      TOKEN_PROGRAM_ID
    );

    const userDepositAccount = await createAssociatedTokenAccount(
      connection,
      baseWallet.payer,
      depositMint,
      tokenOwner,
      undefined,
      TOKEN_PROGRAM_ID
    );

    await mintTo(
      connection,
      baseWallet.payer,
      depositMint,
      userDepositAccount,
      baseWallet.publicKey,
      INITIAL_TOKEN_BALANCE,
      [],
      undefined,
      TOKEN_PROGRAM_ID
    );

    const pool = mock.derivePoolPda(mockProgram.programId, depositMint);
    const lpMint = mock.deriveLpMintPda(mockProgram.programId, pool);
    const vault = mock.deriveVaultAddress(pool, depositMint);

    await mockProgram.methods
      .initializePool()
      .accounts({
        payer: baseWallet.publicKey,
        depositMint,
        depositTokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([baseWallet.payer])
      .rpc()
      .catch((e) => {
        console.log("Error: ", e);
        throw e;
      });

    // LP mint is created by the mock program as a Token-2022 mint
    const userLpAccount = await createAssociatedTokenAccount(
      connection,
      baseWallet.payer,
      lpMint,
      tokenOwner,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );

    return { depositMint, lpMint, vault, userDepositAccount, userLpAccount };
  }

  async function createApprovedSession(
    allowedInstructionsDiscriminators: Buffer,
    discriminatorLen: number
  ): Promise<{ sessionKey: Keypair; userSmartWallet: PublicKey }> {
    const smartWalletOwner = Keypair.generate();
    const sessionKey = Keypair.generate();

    const userSmartWallet = sessionKeys.deriveUserSmartWalletPda(
      program.programId,
      smartWalletOwner.publicKey
    );
    await program.methods
      .createSmartWallet(smartWalletOwner.publicKey)
      .accounts({
        admin: baseWallet.publicKey,
      })
      .signers([baseWallet.payer])
      .rpc()
      .catch((e) => {
        console.log("Error: ", e);
        throw e;
      });

    await program.methods
      .createSession(
        sessionKey.publicKey,
        mockProgram.programId,
        await timestampFromFuture(connection),
        allowedInstructionsDiscriminators,
        discriminatorLen
      )
      .accounts({
        sessionExecutor: baseWallet.publicKey,
        userSmartWallet,
      })
      .signers([baseWallet.payer])
      .rpc()
      .catch((e) => {
        console.log("Error: ", e);
        throw e;
      });

    await program.methods
      .approveSession(sessionKey.publicKey)
      .accounts({
        smartWalletOwner: smartWalletOwner.publicKey,
      })
      .signers([baseWallet.payer, smartWalletOwner])
      .rpc()
      .catch((e) => {
        console.log("Error: ", e);
        throw e;
      });

    return { sessionKey, userSmartWallet };
  }

  function depositInstruction(user: PublicKey, tokenPool: TokenPoolFixture, amount: number) {
    return mockProgram.methods
      .deposit(new anchor.BN(amount))
      // pool seeds reference its own `deposit_mint` field so it cannot be derived by the resolver,
      // the remaining pool accounts are resolved from it through the IDL relations
      .accountsPartial({
        user,
        pool: mock.derivePoolPda(mockProgram.programId, tokenPool.depositMint),
        userDepositAccount: tokenPool.userDepositAccount,
        userLpAccount: tokenPool.userLpAccount,
        depositTokenProgram: TOKEN_PROGRAM_ID,
      })
      .instruction();
  }

  function withdrawInstruction(user: PublicKey, tokenPool: TokenPoolFixture, amount: number) {
    return mockProgram.methods
      .withdraw(new anchor.BN(amount))
      // pool seeds reference its own `deposit_mint` field so it cannot be derived by the resolver,
      // the remaining pool accounts are resolved from it through the IDL relations
      .accountsPartial({
        user,
        pool: mock.derivePoolPda(mockProgram.programId, tokenPool.depositMint),
        userDepositAccount: tokenPool.userDepositAccount,
        userLpAccount: tokenPool.userLpAccount,
        depositTokenProgram: TOKEN_PROGRAM_ID,
      })
      .instruction();
  }

  async function tokenBalances(tokenPool: TokenPoolFixture) {
    const userDepositAccount = await getAccount(
      connection,
      tokenPool.userDepositAccount,
      undefined,
      TOKEN_PROGRAM_ID
    );
    const vault = await getAccount(connection, tokenPool.vault, undefined, TOKEN_PROGRAM_ID);
    const userLpAccount = await getAccount(
      connection,
      tokenPool.userLpAccount,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    const lpMint = await getMint(connection, tokenPool.lpMint, undefined, TOKEN_2022_PROGRAM_ID);

    return {
      userDepositAccount: userDepositAccount.amount,
      vault: vault.amount,
      userLpAccount: userLpAccount.amount,
      lpMintSupply: lpMint.supply,
    };
  }

  it("Execute mock deposit with session executor tokens", async () => {
    const { sessionKey, userSmartWallet } = await createApprovedSession(
      mock.DEPOSIT_DISCRIMINATOR,
      mock.DEPOSIT_DISCRIMINATOR.length
    );

    const sessionExecutor = baseWallet.publicKey;
    const tokenPool = await initializeMockTokenPool(sessionExecutor);
    const depositAccountBefore = await getAccount(
      connection,
      tokenPool.userDepositAccount,
      undefined,
      TOKEN_PROGRAM_ID
    );
    assert.equal(depositAccountBefore.owner.toBase58(), sessionExecutor.toBase58());
    assert.equal(depositAccountBefore.amount, BigInt(INITIAL_TOKEN_BALANCE));

    const depositIx = await depositInstruction(sessionExecutor, tokenPool, DEPOSIT_AMOUNT);
    assert.ok(depositIx.data.subarray(0, 8).equals(mock.DEPOSIT_DISCRIMINATOR));
    // The executor supplies its own tokens. The smart-wallet account is forwarded
    // read-only so the session program can still validate the session's wallet.
    depositIx.keys.push({ pubkey: userSmartWallet, isSigner: false, isWritable: false });

    await executeWithSession(program, {
      sessionExecutor,
      sessionKey: sessionKey.publicKey,
      userSmartWallet,
      targetInstruction: depositIx,
    })
      .signers([baseWallet.payer, sessionKey])
      .rpc()
      .catch((e) => {
        console.log("Error: ", e);
        throw e;
      });

    const balances = await tokenBalances(tokenPool);
    assert.equal(balances.userDepositAccount, BigInt(INITIAL_TOKEN_BALANCE - DEPOSIT_AMOUNT));
    assert.equal(balances.vault, BigInt(DEPOSIT_AMOUNT));
    assert.equal(balances.userLpAccount, BigInt(DEPOSIT_AMOUNT));
    assert.equal(balances.lpMintSupply, BigInt(DEPOSIT_AMOUNT));
  });

  it("Execute mock withdraw not allowed by deposit only session", async () => {
    const { sessionKey, userSmartWallet } = await createApprovedSession(
      mock.DEPOSIT_DISCRIMINATOR,
      mock.DEPOSIT_DISCRIMINATOR.length
    );

    const sessionExecutor = baseWallet.publicKey;
    const tokenPool = await initializeMockTokenPool(sessionExecutor);

    const depositIx = await depositInstruction(sessionExecutor, tokenPool, DEPOSIT_AMOUNT);
    depositIx.keys.push({ pubkey: userSmartWallet, isSigner: false, isWritable: false });
    await executeWithSession(program, {
      sessionExecutor,
      sessionKey: sessionKey.publicKey,
      userSmartWallet,
      targetInstruction: depositIx,
    })
      .signers([baseWallet.payer, sessionKey])
      .rpc()
      .catch((e) => {
        console.log("Error: ", e);
        throw e;
      });

    const balancesBefore = await tokenBalances(tokenPool);

    const withdrawIx = await withdrawInstruction(sessionExecutor, tokenPool, DEPOSIT_AMOUNT);
    assert.ok(withdrawIx.data.subarray(0, 8).equals(mock.WITHDRAW_DISCRIMINATOR));
    assert.ok(!withdrawIx.data.subarray(0, 8).equals(mock.DEPOSIT_DISCRIMINATOR));
    withdrawIx.keys.push({ pubkey: userSmartWallet, isSigner: false, isWritable: false });

    const error = await sendExpectError(
      executeWithSession(program, {
        sessionExecutor,
        sessionKey: sessionKey.publicKey,
        userSmartWallet,
        targetInstruction: withdrawIx,
      })
        .signers([baseWallet.payer, sessionKey])
        .rpc()
    );

    assert.ok(error.includes("NotAllowedInstructionDiscriminator"), error);
    const balancesAfter = await tokenBalances(tokenPool);
    assert.deepEqual(balancesAfter, balancesBefore);
  });
});
