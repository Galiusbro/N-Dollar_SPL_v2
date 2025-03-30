// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import { PublicKey, Keypair } from "@solana/web3.js";
// import {
//   TOKEN_PROGRAM_ID,
//   getAssociatedTokenAddress,
//   ASSOCIATED_TOKEN_PROGRAM_ID,
// } from "@solana/spl-token";
// import { assert } from "chai";
// import { BN } from "bn.js";

// const TOKEN_DECIMALS = 9;

// // Helper function for airdrop
// async function requestAirdrop(
//   provider: anchor.AnchorProvider,
//   address: PublicKey,
//   amount: number
// ) {
//   const signature = await provider.connection.requestAirdrop(
//     address,
//     amount * anchor.web3.LAMPORTS_PER_SOL
//   );
//   const latestBlockhash = await provider.connection.getLatestBlockhash();
//   await provider.connection.confirmTransaction({
//     signature,
//     ...latestBlockhash,
//   });
//   console.log(`Airdropped ${amount} SOL to ${address.toString()}`);
// }

// describe("Token Creation Test", () => {
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//   const tokenCreatorProgram = anchor.workspace.TokenCreator as Program;
//   const wallet = provider.wallet as anchor.Wallet;

//   // Keys and addresses
//   let mint: Keypair;
//   let metadataPda: PublicKey;
//   let tokenInfo: PublicKey;
//   let tokenAccount: PublicKey;

//   before(async () => {
//     console.log("Wallet public key:", wallet.publicKey.toString());

//     mint = Keypair.generate();

//     // Find PDA for metadata
//     [metadataPda] = PublicKey.findProgramAddressSync(
//       [
//         Buffer.from("metadata"),
//         new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBuffer(),
//         mint.publicKey.toBuffer(),
//       ],
//       new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
//     );

//     // Find PDA for token_info
//     [tokenInfo] = PublicKey.findProgramAddressSync(
//       [Buffer.from("token_info"), mint.publicKey.toBuffer()],
//       tokenCreatorProgram.programId
//     );

//     // Get associated token account address
//     tokenAccount = await getAssociatedTokenAddress(
//       mint.publicKey,
//       wallet.publicKey
//     );

//     // Airdrop SOL if needed
//     const balance = await provider.connection.getBalance(wallet.publicKey);
//     if (balance < 2 * anchor.web3.LAMPORTS_PER_SOL) {
//       await requestAirdrop(provider, wallet.publicKey, 2);
//     }
//   });

//   it("Creates a new token", async () => {
//     try {
//       const totalSupply = new BN(1_000_000_000).mul(
//         new BN(10).pow(new BN(TOKEN_DECIMALS))
//       );

//       const tx = await tokenCreatorProgram.methods
//         .createUserToken(
//           "Test Token",
//           "TEST",
//           "https://test.com/token.json",
//           TOKEN_DECIMALS,
//           totalSupply
//         )
//         .accounts({
//           mint: mint.publicKey,
//           metadata: metadataPda,
//           tokenInfo: tokenInfo,
//           authority: wallet.publicKey,
//           tokenAccount: tokenAccount,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//           systemProgram: anchor.web3.SystemProgram.programId,
//           rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//           tokenMetadataProgram: new PublicKey(
//             "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
//           ),
//         })
//         .preInstructions([
//           anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
//             units: 400_000,
//           }),
//         ])
//         .signers([mint])
//         .rpc();

//       console.log("Create token tx:", tx);

//       // Check token balance
//       const balance = await provider.connection.getTokenAccountBalance(
//         tokenAccount
//       );
//       console.log("Token balance:", balance.value.uiAmountString);

//       // Assert we have the full supply
//       assert.equal(
//         balance.value.amount,
//         totalSupply.toString(),
//         "Token account should have the full supply"
//       );

//       // Check token info account
//       const tokenInfoAccount = await tokenCreatorProgram.account[
//         "tokenInfo"
//       ].fetch(tokenInfo);
//       console.log("Token info:", {
//         mint: tokenInfoAccount.mint.toString(),
//         authority: tokenInfoAccount.authority.toString(),
//         totalSupply: tokenInfoAccount.totalSupply.toString(),
//       });

//       // Assert token info is correct
//       assert.equal(
//         tokenInfoAccount.mint.toString(),
//         mint.publicKey.toString(),
//         "Token info mint should match created mint"
//       );

//       assert.equal(
//         tokenInfoAccount.authority.toString(),
//         wallet.publicKey.toString(),
//         "Token info authority should match wallet"
//       );

//       assert.equal(
//         tokenInfoAccount.totalSupply.toString(),
//         totalSupply.toString(),
//         "Token info supply should match created supply"
//       );
//     } catch (error) {
//       console.error("Error creating token:", error);
//       throw error;
//     }
//   });
// });
