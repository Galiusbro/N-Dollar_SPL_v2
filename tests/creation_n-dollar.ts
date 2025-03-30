// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import { PublicKey, Keypair } from "@solana/web3.js";
// import {
//   TOKEN_PROGRAM_ID,
//   createAssociatedTokenAccount,
//   createAssociatedTokenAccountInstruction,
// } from "@solana/spl-token";
// import { assert } from "chai";
// import { BN } from "bn.js";

// describe("N-Dollar Token Creation Test", () => {
//   // Program and account setup
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//   const NdollarProgram = anchor.workspace.NDollar as Program;
//   const liquidityPoolProgram = anchor.workspace.LiquidityPool as Program;
//   const genesisProgram = anchor.workspace.TokenCreator as Program;
//   const wallet = provider.wallet as anchor.Wallet;

//   const METADATA_PROGRAM_ID = new PublicKey(
//     "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
//   );
//   const WRAPPED_SOL_MINT = new PublicKey(
//     "So11111111111111111111111111111111111111112"
//   );
//   const SOL_DECIMALS = anchor.web3.LAMPORTS_PER_SOL;

//   // Test keypairs and PDAs
//   const n_dollar_mint = Keypair.generate();
//   let metadataPda, poolPda, solVaultPda, nDollarVault, userNDollarAccount;

//   // Utility functions
//   async function getOrCreateATA(mint, owner, payer) {
//     const ata = await anchor.utils.token.associatedAddress({ mint, owner });

//     try {
//       await provider.connection.getTokenAccountBalance(ata);
//       console.log("ATA exists:", ata.toString());
//     } catch {
//       try {
//         const createATAIx = createAssociatedTokenAccountInstruction(
//           "payer" in payer ? payer.payer.publicKey : payer.publicKey,
//           ata,
//           owner,
//           mint
//         );
//         const tx = new anchor.web3.Transaction().add(createATAIx);
//         await provider.sendAndConfirm(tx);
//         console.log("Created new ATA:", ata.toString());
//       } catch (e) {
//         console.error("Error creating ATA:", e);
//       }
//     }
//     return ata;
//   }

//   async function airdropSol(address, amount) {
//     const signature = await provider.connection.requestAirdrop(address, amount);
//     await provider.connection.confirmTransaction(signature);
//     console.log(
//       `Airdropped ${amount / SOL_DECIMALS} SOL to ${address.toString()}`
//     );
//   }

//   function readMetaplexString(buffer, offset) {
//     const length = buffer.readUInt32LE(offset);
//     if (length === 0) return ["", offset + 4];

//     const str = buffer.slice(offset + 4, offset + 4 + length).toString("utf8");
//     return [str, offset + 4 + length];
//   }

//   function parseMetadata(data) {
//     try {
//       let offset = 1 + 32 + 32; // Skip past header
//       const [name, nameEnd] = readMetaplexString(data, offset);
//       const [symbol, symbolEnd] = readMetaplexString(data, nameEnd);
//       const [uri] = readMetaplexString(data, symbolEnd);

//       return { name, symbol, uri };
//     } catch (error) {
//       console.error("Error parsing metadata:", error);
//       return null;
//     }
//   }

//   function logTokenInfo(name, metadata, balance = null) {
//     console.log(`\n${name} Token Info:`);
//     if (metadata) {
//       console.log("Name:", metadata.name);
//       console.log("Symbol:", metadata.symbol);
//       console.log("URI:", metadata.uri);
//     }
//     if (balance !== null) {
//       console.log("Balance:", balance);
//     }
//   }

//   // Initialize PDAs and accounts before tests
//   before(async () => {
//     // Find program addresses
//     [metadataPda] = PublicKey.findProgramAddressSync(
//       [
//         Buffer.from("metadata"),
//         METADATA_PROGRAM_ID.toBuffer(),
//         n_dollar_mint.publicKey.toBuffer(),
//       ],
//       METADATA_PROGRAM_ID
//     );

//     [poolPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("pool"), n_dollar_mint.publicKey.toBuffer()],
//       liquidityPoolProgram.programId
//     );

//     [solVaultPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("sol_vault"), poolPda.toBuffer()],
//       liquidityPoolProgram.programId
//     );

//     nDollarVault = await anchor.utils.token.associatedAddress({
//       mint: n_dollar_mint.publicKey,
//       owner: poolPda,
//     });

//     userNDollarAccount = await anchor.utils.token.associatedAddress({
//       mint: n_dollar_mint.publicKey,
//       owner: wallet.publicKey,
//     });

//     // Ensure wallet has enough SOL
//     const userBalance = await provider.connection.getBalance(wallet.publicKey);
//     if (userBalance < 10000 * SOL_DECIMALS) {
//       await airdropSol(wallet.publicKey, 10000 * SOL_DECIMALS);
//     }

//     console.log(
//       "User SOL balance:",
//       (await provider.connection.getBalance(wallet.publicKey)) / SOL_DECIMALS,
//       "SOL"
//     );
//   });

//   it("Creates token with metadata", async () => {
//     const tx = await NdollarProgram.methods
//       .createToken("One-Click Token", "ONE", "https://oneclick.com/token.json")
//       .accounts({
//         mint: n_dollar_mint.publicKey,
//         metadata: metadataPda,
//         authority: wallet.publicKey,
//         tokenProgram: TOKEN_PROGRAM_ID,
//         systemProgram: anchor.web3.SystemProgram.programId,
//         rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//         tokenMetadataProgram: METADATA_PROGRAM_ID,
//       })
//       .signers([n_dollar_mint])
//       .rpc();

//     console.log("Create N-Dollar token tx:", tx);
//     console.log("N-Dollar mint pubkey:", n_dollar_mint.publicKey.toString());

//     const mintInfo = await provider.connection.getAccountInfo(
//       n_dollar_mint.publicKey
//     );
//     assert(mintInfo !== null, "N-Dollar mint not created");

//     const metadataInfo = await provider.connection.getAccountInfo(metadataPda);
//     assert(metadataInfo !== null, "Metadata account not found");

//     const metadata = parseMetadata(metadataInfo.data);
//     logTokenInfo("N-Dollar", metadata);
//   });

//   it("Initializes liquidity pool", async () => {
//     const tx = await NdollarProgram.methods
//       .initializeLiquidityPool()
//       .accounts({
//         mint: n_dollar_mint.publicKey,
//         pool: poolPda,
//         ndollarVault: nDollarVault,
//         solVault: solVaultPda,
//         authority: wallet.publicKey,
//         tokenProgram: TOKEN_PROGRAM_ID,
//         systemProgram: anchor.web3.SystemProgram.programId,
//         associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
//         rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//         liquidityPoolProgram: liquidityPoolProgram.programId,
//       })
//       .rpc();

//     console.log("Initialize liquidity pool tx:", tx);

//     const nDollarVaultAccount =
//       await provider.connection.getTokenAccountBalance(nDollarVault);
//     console.log("N-Dollar vault balance:", nDollarVaultAccount.value.uiAmount);

//     const expectedAmount = 108_000_000;
//     assert.equal(
//       nDollarVaultAccount.value.uiAmount,
//       expectedAmount,
//       `Expected ${expectedAmount} tokens in vault, but got ${nDollarVaultAccount.value.uiAmount}`
//     );

//     const solVaultBalance = await provider.connection.getBalance(solVaultPda);
//     console.log("SOL vault balance:", solVaultBalance / SOL_DECIMALS, "SOL");
//   });

//   it("Adds initial liquidity", async () => {
//     await getOrCreateATA(n_dollar_mint.publicKey, wallet.publicKey, wallet);
//     const solAmount = new BN(10000 * SOL_DECIMALS);

//     const tx = await liquidityPoolProgram.methods
//       .addLiquidity(new BN(0), solAmount)
//       .accounts({
//         pool: poolPda,
//         ndollarMint: n_dollar_mint.publicKey,
//         ndollarVault: nDollarVault,
//         solVault: solVaultPda,
//         user: wallet.publicKey,
//         userNdollar: userNDollarAccount,
//         systemProgram: anchor.web3.SystemProgram.programId,
//         tokenProgram: TOKEN_PROGRAM_ID,
//       })
//       .rpc();

//     console.log("Add liquidity tx:", tx);

//     const nDollarVaultBalance =
//       await provider.connection.getTokenAccountBalance(nDollarVault);
//     console.log("Pool N-Dollar balance:", nDollarVaultBalance.value.uiAmount);

//     const solVaultBalance = await provider.connection.getBalance(solVaultPda);
//     console.log("Pool SOL balance:", solVaultBalance / SOL_DECIMALS, "SOL");
//   });

//   it("Swaps SOL to N-Dollar", async () => {
//     const solAmount = new BN(1 * SOL_DECIMALS);

//     const tx = await liquidityPoolProgram.methods
//       .swapSolToNdollar(solAmount)
//       .accounts({
//         pool: poolPda,
//         ndollarMint: n_dollar_mint.publicKey,
//         ndollarVault: nDollarVault,
//         solVault: solVaultPda,
//         user: wallet.publicKey,
//         userNdollar: userNDollarAccount,
//         systemProgram: anchor.web3.SystemProgram.programId,
//         tokenProgram: TOKEN_PROGRAM_ID,
//       })
//       .rpc();

//     console.log("Swap SOL to N-Dollar tx:", tx);

//     const userNDollarBalance = await provider.connection.getTokenAccountBalance(
//       userNDollarAccount
//     );
//     console.log(
//       "User N-Dollar balance after swap:",
//       userNDollarBalance.value.uiAmount
//     );

//     const poolSolBalance = await provider.connection.getBalance(solVaultPda);
//     console.log(
//       "Pool SOL balance after swap:",
//       poolSolBalance / SOL_DECIMALS,
//       "SOL"
//     );
//   });

//   it("Swaps N-Dollar to SOL", async () => {
//     const userNDollarBalance = await provider.connection.getTokenAccountBalance(
//       userNDollarAccount
//     );
//     const ndollarAmount = new BN(userNDollarBalance.value.amount).divn(2);
//     const userSolBalanceBefore = await provider.connection.getBalance(
//       wallet.publicKey
//     );

//     const tx = await liquidityPoolProgram.methods
//       .swapNdollarToSol(ndollarAmount)
//       .accounts({
//         pool: poolPda,
//         ndollarMint: n_dollar_mint.publicKey,
//         ndollarVault: nDollarVault,
//         solVault: solVaultPda,
//         user: wallet.publicKey,
//         userNdollar: userNDollarAccount,
//         systemProgram: anchor.web3.SystemProgram.programId,
//         tokenProgram: TOKEN_PROGRAM_ID,
//       })
//       .rpc();

//     console.log("Swap N-Dollar to SOL tx:", tx);

//     const userNDollarBalanceAfter =
//       await provider.connection.getTokenAccountBalance(userNDollarAccount);
//     console.log(
//       "User N-Dollar balance after swap:",
//       userNDollarBalanceAfter.value.uiAmount
//     );

//     const userSolBalanceAfter = await provider.connection.getBalance(
//       wallet.publicKey
//     );
//     const solReceived =
//       (userSolBalanceAfter - userSolBalanceBefore) / SOL_DECIMALS;
//     console.log("User received SOL:", solReceived, "SOL");

//     const poolSolBalance = await provider.connection.getBalance(solVaultPda);
//     console.log(
//       "Pool SOL balance after swap:",
//       poolSolBalance / SOL_DECIMALS,
//       "SOL"
//     );
//   });

//   it("Creates user token using N-Dollars", async () => {
//     const userTokenMint = Keypair.generate();
//     console.log("\nCreating user token...");
//     console.log("User token mint pubkey:", userTokenMint.publicKey.toString());

//     const [userTokenMetadataPda] = PublicKey.findProgramAddressSync(
//       [
//         Buffer.from("metadata"),
//         METADATA_PROGRAM_ID.toBuffer(),
//         userTokenMint.publicKey.toBuffer(),
//       ],
//       METADATA_PROGRAM_ID
//     );

//     const [tokenInfoPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("token_info"), userTokenMint.publicKey.toBuffer()],
//       genesisProgram.programId
//     );

//     const userTokenAccount = await anchor.utils.token.associatedAddress({
//       mint: userTokenMint.publicKey,
//       owner: wallet.publicKey,
//     });

//     try {
//       const tx = await genesisProgram.methods
//         .createUserToken(
//           "User Token",
//           "USER",
//           "https://example.com/user-token.json",
//           9,
//           new BN(1000000000),
//           new BN(10258320)
//         )
//         .accounts({
//           mint: userTokenMint.publicKey,
//           metadata: userTokenMetadataPda,
//           tokenInfo: tokenInfoPda,
//           authority: wallet.publicKey,
//           tokenAccount: userTokenAccount,
//           liquidityPool: poolPda,
//           poolNDollarAccount: nDollarVault,
//           poolSolAccount: solVaultPda,
//           userNDollarAccount: userNDollarAccount,
//           nDollarMint: n_dollar_mint.publicKey,
//           solMint: WRAPPED_SOL_MINT,
//           liquidityPoolProgram: liquidityPoolProgram.programId,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
//           systemProgram: anchor.web3.SystemProgram.programId,
//           rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//           tokenMetadataProgram: METADATA_PROGRAM_ID,
//         })
//         .signers([userTokenMint])
//         .rpc();

//       console.log("Create user token tx:", tx);

//       const userTokenBalance = await provider.connection.getTokenAccountBalance(
//         userTokenAccount
//       );
//       const metadataInfo = await provider.connection.getAccountInfo(
//         userTokenMetadataPda
//       );
//       const metadata = metadataInfo ? parseMetadata(metadataInfo.data) : null;

//       logTokenInfo("User", metadata, userTokenBalance.value.uiAmount);
//     } catch (error) {
//       console.error("Error creating user token:", error);
//       throw error;
//     }
//   });
// });
