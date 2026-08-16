import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { PublicKey } from "@solana/web3.js";
import assert from "assert";

import { R3SessionKeys } from "../target/types/r3_session_keys";
import * as sessionKeys from "./utils/pda";
import { ensureProgramInitialized } from "./utils/helpers";

describe("00 - Initialize", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const baseWallet = provider.wallet as anchor.Wallet;

  const program = anchor.workspace.r3SessionKeys as Program<R3SessionKeys>;

  let programConfigPda: PublicKey;
  let programConfigBump: number;

  before(async () => {
    [programConfigPda, programConfigBump] = sessionKeys.findProgramConfigPda(program.programId);
    console.log("Program id: ", program.programId.toBase58());
    console.log("Admin wallet: ", baseWallet.publicKey.toBase58());
  });

  it("Initialize program config", async () => {
    await ensureProgramInitialized(program, baseWallet);

    const programConfig = await program.account.programConfig.fetch(programConfigPda);
    assert.equal(programConfig.admin.toBase58(), baseWallet.publicKey.toBase58());
    assert.ok("active" in programConfig.status);
    assert.equal(programConfig.bump, programConfigBump);
  });
});
