// import * as anchor from "@coral-xyz/anchor";
// import { Program, web3, AnchorProvider } from "@coral-xyz/anchor";
// import {
//   createMint,
//   getOrCreateAssociatedTokenAccount,
//   mintTo,
//   createAssociatedTokenAccount,
//   TOKEN_PROGRAM_ID,
//   ASSOCIATED_TOKEN_PROGRAM_ID,
// } from "@solana/spl-token";
// import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
// import { BN } from "bn.js";

// describe("N-Dollar Token Creation Test", () => {
//   const provider = AnchorProvider.env();
//   anchor.setProvider(provider);
//   const wallet = provider.wallet as anchor.Wallet;

//   const nDollarProgram = anchor.workspace.NDollar as Program;
//   const liquidityProgram = anchor.workspace.LiquidityPool as Program;
//   const genesisProgram = anchor.workspace.TokenConsumer as Program;
//   const METADATA_PROGRAM_ID = new PublicKey(
//     "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
//   );

//   const readStr = (buf: Buffer, offset: number): [string, number] => {
//     const len = buf.readUInt32LE(offset);
//     const str = buf.slice(offset + 4, offset + 4 + len).toString("utf8");
//     return [str, offset + 4 + len];
//   };

//   async function airdropIfNeeded(pubkey: PublicKey, min = 1) {
//     const balance = await provider.connection.getBalance(pubkey);
//     if (balance < min * web3.LAMPORTS_PER_SOL) {
//       const sig = await provider.connection.requestAirdrop(
//         pubkey,
//         2 * web3.LAMPORTS_PER_SOL
//       );
//       await provider.connection.confirmTransaction(sig);
//     }
//   }

//   it("🎯 Full pipeline: Create token, init pool, swap via PDA", async () => {
//     await airdropIfNeeded(wallet.publicKey);

//     // === Create mint + metadata ===
//     const mint = Keypair.generate();

//     const [metadataPda] = PublicKey.findProgramAddressSync(
//       [
//         Buffer.from("metadata"),
//         METADATA_PROGRAM_ID.toBuffer(),
//         mint.publicKey.toBuffer(),
//       ],
//       METADATA_PROGRAM_ID
//     );

//     await nDollarProgram.methods
//       .createToken("OneClick", "ONE", "https://oneclick.com/token.json")
//       .accounts({
//         mint: mint.publicKey,
//         metadata: metadataPda,
//         authority: wallet.publicKey,
//         tokenProgram: TOKEN_PROGRAM_ID,
//         systemProgram: SystemProgram.programId,
//         rent: web3.SYSVAR_RENT_PUBKEY,
//         tokenMetadataProgram: METADATA_PROGRAM_ID,
//       })
//       .signers([mint])
//       .rpc();

//     console.log("✅ Token created");

//     console.log("Liquidity Program ID:", liquidityProgram.programId.toString());
//     console.log("N-Dollar Program ID:", nDollarProgram.programId.toString());

//     // === Derive PDAs ===
//     const [poolPda, poolBump] = PublicKey.findProgramAddressSync(
//       [Buffer.from("pool"), mint.publicKey.toBuffer()],
//       liquidityProgram.programId
//     );

//     // Debug: Check PDAs with different program IDs
//     const programs = [
//       { name: "Liquidity Program", id: liquidityProgram.programId },
//       { name: "N-Dollar Program", id: nDollarProgram.programId },
//       {
//         name: "Hardcoded ID",
//         id: new PublicKey("B24yupzEDjF7Z9frnDG16uAwH1ZYfB57kuzh8jwDsL83"),
//       },
//     ];

//     for (const program of programs) {
//       const [testPda] = PublicKey.findProgramAddressSync(
//         [Buffer.from("sol_vault"), poolPda.toBuffer()],
//         program.id
//       );
//       console.log(`${program.name} sol_vault PDA:`, testPda.toString());
//     }

//     const [solVaultPda, solVaultBump] = PublicKey.findProgramAddressSync(
//       [Buffer.from("sol_vault"), poolPda.toBuffer()],
//       liquidityProgram.programId
//     );

//     // === Create ndollar_vault (ATA for pool) ===
//     const poolNdollarVault = await createAssociatedTokenAccount(
//       provider.connection,
//       wallet.payer,
//       mint.publicKey,
//       poolPda,
//       undefined,
//       TOKEN_PROGRAM_ID,
//       ASSOCIATED_TOKEN_PROGRAM_ID,
//       true
//     );

//     // === Init liquidity pool via CPI ===
//     await nDollarProgram.methods
//       .initializeLiquidityPool()
//       .accounts({
//         mint: mint.publicKey,
//         pool: poolPda,
//         ndollarVault: poolNdollarVault,
//         solVault: solVaultPda,
//         authority: wallet.publicKey,
//         tokenProgram: TOKEN_PROGRAM_ID,
//         systemProgram: SystemProgram.programId,
//         rent: web3.SYSVAR_RENT_PUBKEY,
//         liquidityPoolProgram: liquidityProgram.programId,
//       })
//       .rpc();

//     console.log("✅ Pool initialized");

//     // === Create PDA for token creation vault ===

//     // Debug: Try different seed combinations
//     console.log("\nDebug: Trying different seed combinations:");

//     const seedCombinations = [
//       {
//         name: "token_creation_vault",
//         seeds: [
//           Buffer.from("token_creation_vault"),
//           wallet.publicKey.toBuffer(),
//         ],
//       },
//       {
//         name: "token_vault",
//         seeds: [Buffer.from("token_vault"), wallet.publicKey.toBuffer()],
//       },
//       {
//         name: "vault",
//         seeds: [Buffer.from("vault"), wallet.publicKey.toBuffer()],
//       },
//       {
//         name: "user",
//         seeds: [Buffer.from("user"), wallet.publicKey.toBuffer()],
//       },
//     ];

//     for (const program of programs) {
//       console.log(`\nTesting with ${program.name}:`);
//       for (const seedCombo of seedCombinations) {
//         const [testPda] = PublicKey.findProgramAddressSync(
//           seedCombo.seeds,
//           program.id
//         );
//         console.log(`  ${seedCombo.name}:`, testPda.toString());
//       }
//     }

//     console.log("\nTesting with Genesis Program:");
//     for (const seedCombo of seedCombinations) {
//       const [testPda] = PublicKey.findProgramAddressSync(
//         seedCombo.seeds,
//         genesisProgram.programId
//       );
//       console.log(`  ${seedCombo.name}:`, testPda.toString());
//     }

//     const [vaultPda, vaultBump] = PublicKey.findProgramAddressSync(
//       [Buffer.from("token_creation_vault"), wallet.publicKey.toBuffer()],
//       genesisProgram.programId
//     );

//     console.log("\nFinal values being used:");
//     console.log("Genesis Program ID:", genesisProgram.programId.toString());
//     console.log("Actual vault PDA being used:", vaultPda.toString());
//     console.log("Wallet public key:", wallet.publicKey.toString());
//     console.log(
//       "Expected PDA from error:",
//       "28URBueGhnif6Lf5L9ujDd7aJcW4RV4sGPf3MJ2HmAHp"
//     );

//     const vaultAta = await getOrCreateAssociatedTokenAccount(
//       provider.connection,
//       wallet.payer,
//       mint.publicKey,
//       vaultPda,
//       true
//     );

//     // === Mint 10 N-Dollar to PDA
//     await mintTo(
//       provider.connection,
//       wallet.payer,
//       mint.publicKey,
//       vaultAta.address,
//       wallet.payer,
//       10_000_000_000 // 10 N$
//     );

//     // === Swap via PDA ===
//     console.log("\nVerifying PDAs before swap:");
//     console.log("Pool PDA:", poolPda.toString());
//     console.log("Pool bump:", poolBump);
//     console.log("SOL Vault PDA:", solVaultPda.toString());
//     console.log("SOL Vault bump:", solVaultBump);
//     console.log("Token Creation Vault PDA:", vaultPda.toString());
//     console.log("Token Creation Vault bump:", vaultBump);
//     console.log("Token Creation Vault ATA:", vaultAta.address.toString());
//     console.log("Pool N-Dollar Vault:", poolNdollarVault.toString());

//     // Check balances before swap
//     const pdaSolBefore = await provider.connection.getBalance(vaultPda);
//     const vaultAtaBalanceBefore = (
//       await provider.connection.getTokenAccountBalance(vaultAta.address)
//     ).value.amount;
//     console.log(
//       "💰 PDA SOL balance before:",
//       pdaSolBefore / web3.LAMPORTS_PER_SOL
//     );
//     console.log("💰 PDA N-Dollar balance before:", vaultAtaBalanceBefore);

//     const tx = await genesisProgram.methods
//       .triggerSwap(new BN(5_000_000_000), vaultBump)
//       .accounts({
//         authority: wallet.publicKey,
//         tokenCreationVault: vaultPda,
//         tokenCreationVaultNdollar: vaultAta.address,
//         pool: poolPda,
//         ndollarMint: mint.publicKey,
//         ndollarVault: poolNdollarVault,
//         solVault: solVaultPda,
//         liquidityPoolProgram: liquidityProgram.programId,
//         systemProgram: SystemProgram.programId,
//         tokenProgram: TOKEN_PROGRAM_ID,
//       })
//       .rpc();

//     console.log("✅ Swap via PDA done:", tx);

//     const pdaSol = await provider.connection.getBalance(vaultPda);
//     const vaultAtaBalance = (
//       await provider.connection.getTokenAccountBalance(vaultAta.address)
//     ).value.amount;
//     console.log("💰 PDA SOL balance after:", pdaSol / web3.LAMPORTS_PER_SOL);
//     console.log("💰 PDA N-Dollar balance after:", vaultAtaBalance);
//   });
// });
