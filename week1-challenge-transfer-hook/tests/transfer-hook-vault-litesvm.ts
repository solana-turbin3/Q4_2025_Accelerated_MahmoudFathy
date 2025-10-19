import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TransferHookVaultLitesvm } from "../target/types/transfer_hook_vault_litesvm";

describe("transfer-hook-vault-litesvm", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.transferHookVaultLitesvm as Program<TransferHookVaultLitesvm>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
