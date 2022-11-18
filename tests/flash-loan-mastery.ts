import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { FlashLoanMastery } from "../target/types/flash_loan_mastery";

describe("flash-loan-mastery", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.FlashLoanMastery as Program<FlashLoanMastery>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
