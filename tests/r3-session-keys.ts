import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { assert } from "chai";
import { R3SessionKeys } from "../target/types/r3_session_keys";

describe("r3-session-keys", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.r3SessionKeys as Program<R3SessionKeys>;

  it("Initializes and increments a counter", async () => {
    const [counter] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("counter")],
      program.programId
    );

    const initializeTx = await program.methods
      .initialize()
      .accountsPartial({ counter })
      .rpc();
    console.log("Initialize transaction signature", initializeTx);

    let counterAccount = await program.account.counter.fetch(counter);
    assert.equal(counterAccount.count.toNumber(), 0);
    assert.equal(
      counterAccount.authority.toBase58(),
      provider.wallet.publicKey.toBase58()
    );

    const incrementTx = await program.methods
      .increment()
      .accountsPartial({ counter })
      .rpc();
    console.log("Increment transaction signature", incrementTx);

    counterAccount = await program.account.counter.fetch(counter);
    assert.equal(counterAccount.count.toNumber(), 1);
    assert.equal(
      counterAccount.authority.toBase58(),
      provider.wallet.publicKey.toBase58()
    );
  });
});
