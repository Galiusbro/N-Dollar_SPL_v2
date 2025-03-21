import * as anchor from "@coral-xyz/anchor";
import { AnchorProvider } from "@coral-xyz/anchor";
import { Keypair } from "@solana/web3.js";

/**
 * Sets up test accounts with SOL
 */
export async function setupTestAccounts(
  provider: AnchorProvider,
  admin: Keypair,
  user1: Keypair,
  user2: Keypair
): Promise<void> {
  // Airdrop SOL to test accounts
  await provider.connection.requestAirdrop(
    admin.publicKey,
    10 * anchor.web3.LAMPORTS_PER_SOL
  );
  await provider.connection.requestAirdrop(
    user1.publicKey,
    5 * anchor.web3.LAMPORTS_PER_SOL
  );
  await provider.connection.requestAirdrop(
    user2.publicKey,
    5 * anchor.web3.LAMPORTS_PER_SOL
  );

  // Wait for confirmations
  await new Promise((resolve) => setTimeout(resolve, 2000));

  console.log("Test accounts successfully initialized");
  console.log("Admin:", admin.publicKey.toString());
  console.log("User1:", user1.publicKey.toString());
  console.log("User2:", user2.publicKey.toString());
}
