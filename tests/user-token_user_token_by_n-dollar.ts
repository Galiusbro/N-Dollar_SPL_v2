// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import { PublicKey, Keypair, Transaction } from "@solana/web3.js";
// import {
//   TOKEN_PROGRAM_ID,
//   getAssociatedTokenAddress,
//   ASSOCIATED_TOKEN_PROGRAM_ID,
//   createAssociatedTokenAccountInstruction,
// } from "@solana/spl-token";
// import { assert } from "chai";
// import { BN } from "bn.js";

// describe("Token Creation with N-Dollar Test", () => {
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//   // Программы
//   const nDollarProgram = anchor.workspace.NDollar as Program;
//   const tokenCreatorProgram = anchor.workspace.TokenCreator as Program;
//   const liquidityPoolProgram = anchor.workspace.LiquidityPool as Program;
//   const wallet = provider.wallet as anchor.Wallet;

//   // Ключи и адреса для N-доллара
//   let nDollarMint: Keypair;
//   let nDollarMetadataPda: PublicKey;
//   let nDollarPoolAccount: PublicKey;
//   let nDollarSolAccount: PublicKey;
//   let userNDollarAccount: PublicKey;
//   let poolAuthorityPda: PublicKey;
//   let mintSchedulePda: PublicKey;

//   // Ключи и адреса для создаваемого токена
//   let newTokenMint: Keypair;
//   let newTokenMetadataPda: PublicKey;
//   let newTokenInfo: PublicKey;
//   let userNewTokenAccount: PublicKey;

//   const TOKEN_DECIMALS = 9;

//   // Добавляем функцию для вывода состояния пула
//   async function logPoolState(message: string) {
//     const poolTokenBalance = await provider.connection.getTokenAccountBalance(
//       nDollarPoolAccount
//     );
//     const solBalance = await provider.connection.getBalance(nDollarSolAccount);
//     const price = await liquidityPoolProgram.methods
//       .getPrice()
//       .accounts({
//         poolAccount: nDollarPoolAccount,
//         solAccount: nDollarSolAccount,
//       })
//       .view();

//     console.log("\n=== " + message + " ===");
//     console.log(
//       "Pool N-Dollars:",
//       Number(poolTokenBalance.value.uiAmountString).toLocaleString()
//     );
//     console.log(
//       "Pool SOL:",
//       (solBalance / anchor.web3.LAMPORTS_PER_SOL).toLocaleString(),
//       "SOL"
//     );
//     const priceStr = new BN(price.toString()).toString();
//     console.log("Token price:", priceStr, "tokens per LAMPORT");
//     const pricePerSol = Number(priceStr) * 1_000_000_000;
//     console.log("Token price:", pricePerSol.toLocaleString(), "tokens per SOL");
//     console.log("===============\n");
//   }

//   // Добавляем функцию для вывода баланса пользователя
//   async function logUserBalance(message: string) {
//     const userSolBalance = await provider.connection.getBalance(
//       wallet.publicKey
//     );
//     let userNDollarBalance;
//     try {
//       userNDollarBalance = await provider.connection.getTokenAccountBalance(
//         userNDollarAccount
//       );
//     } catch {
//       userNDollarBalance = { value: { uiAmountString: "0" } };
//     }

//     console.log("\n=== " + message + " ===");
//     console.log(
//       "User SOL:",
//       (userSolBalance / anchor.web3.LAMPORTS_PER_SOL).toLocaleString(),
//       "SOL"
//     );
//     console.log(
//       "User N-Dollars:",
//       Number(userNDollarBalance.value.uiAmountString).toLocaleString()
//     );
//     console.log("===============\n");
//   }

//   before(async () => {
//     console.log("Initializing test accounts...");

//     // Генерируем ключи для N-доллара
//     nDollarMint = Keypair.generate();
//     newTokenMint = Keypair.generate();

//     // Находим PDA для метаданных N-доллара
//     [nDollarMetadataPda] = PublicKey.findProgramAddressSync(
//       [
//         Buffer.from("metadata"),
//         new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBuffer(),
//         nDollarMint.publicKey.toBuffer(),
//       ],
//       new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
//     );

//     // Находим PDA для метаданных нового токена
//     [newTokenMetadataPda] = PublicKey.findProgramAddressSync(
//       [
//         Buffer.from("metadata"),
//         new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBuffer(),
//         newTokenMint.publicKey.toBuffer(),
//       ],
//       new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
//     );

//     // Находим PDA для token_info
//     [newTokenInfo] = PublicKey.findProgramAddressSync(
//       [Buffer.from("token_info"), newTokenMint.publicKey.toBuffer()],
//       tokenCreatorProgram.programId
//     );

//     // Находим PDA для пула ликвидности
//     [poolAuthorityPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("pool_authority")],
//       liquidityPoolProgram.programId
//     );

//     [nDollarSolAccount] = PublicKey.findProgramAddressSync(
//       [Buffer.from("sol_account"), nDollarMint.publicKey.toBuffer()],
//       liquidityPoolProgram.programId
//     );

//     // Находим PDA для mint_schedule
//     [mintSchedulePda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("mint_schedule"), nDollarMint.publicKey.toBuffer()],
//       nDollarProgram.programId
//     );

//     // Получаем адреса ассоциированных токен-аккаунтов
//     nDollarPoolAccount = await getAssociatedTokenAddress(
//       nDollarMint.publicKey,
//       poolAuthorityPda,
//       true
//     );

//     userNDollarAccount = await getAssociatedTokenAddress(
//       nDollarMint.publicKey,
//       wallet.publicKey
//     );

//     userNewTokenAccount = await getAssociatedTokenAddress(
//       newTokenMint.publicKey,
//       wallet.publicKey
//     );

//     // Аирдроп SOL если нужно
//     const balance = await provider.connection.getBalance(wallet.publicKey);
//     if (balance < 2 * anchor.web3.LAMPORTS_PER_SOL) {
//       const signature = await provider.connection.requestAirdrop(
//         wallet.publicKey,
//         2 * anchor.web3.LAMPORTS_PER_SOL
//       );
//       await provider.connection.confirmTransaction(signature);
//     }
//   });

//   it("Creates N-Dollar token and initializes liquidity pool", async () => {
//     try {
//       const tx = await nDollarProgram.methods
//         .createToken(
//           "N-Dollar",
//           "NDOL",
//           "https://test.com/ndollar.json",
//           TOKEN_DECIMALS
//         )
//         .accounts({
//           mint: nDollarMint.publicKey,
//           metadata: nDollarMetadataPda,
//           mintSchedule: mintSchedulePda,
//           authority: wallet.publicKey,
//           poolAccount: nDollarPoolAccount,
//           solAccount: nDollarSolAccount,
//           poolAuthority: poolAuthorityPda,
//           liquidityPoolProgram: liquidityPoolProgram.programId,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//           systemProgram: anchor.web3.SystemProgram.programId,
//           rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//           tokenMetadataProgram: new PublicKey(
//             "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
//           ),
//         })
//         .signers([nDollarMint])
//         .rpc();

//       console.log("N-Dollar creation tx:", tx);
//       await logPoolState("Initial pool state after creation");

//       // Проверяем баланс пула
//       const poolBalance = await provider.connection.getTokenAccountBalance(
//         nDollarPoolAccount
//       );
//       console.log("Pool N-Dollar balance:", poolBalance.value.uiAmountString);
//       assert.equal(poolBalance.value.uiAmount, 108_000_000);
//     } catch (error) {
//       console.error("Error creating N-Dollar:", error);
//       throw error;
//     }
//   });

//   it("Buys N-Dollars from liquidity pool", async () => {
//     try {
//       // Создаем ATA для пользователя
//       const createAtaIx = createAssociatedTokenAccountInstruction(
//         wallet.publicKey,
//         userNDollarAccount,
//         wallet.publicKey,
//         nDollarMint.publicKey
//       );

//       try {
//         await provider.sendAndConfirm(new Transaction().add(createAtaIx));
//         console.log(
//           "Created user N-Dollar ATA:",
//           userNDollarAccount.toString()
//         );
//       } catch (e) {
//         console.log("ATA already exists");
//       }

//       // Логируем начальное состояние
//       await logUserBalance("Initial user state");
//       await logPoolState("Initial pool state");

//       // Покупаем N-доллары за 0.1 SOL
//       const swapAmount = new BN(100_000_000); // 0.1 SOL
//       console.log(
//         "\nSwapping",
//         swapAmount.toNumber() / 1_000_000_000,
//         "SOL to N-Dollars"
//       );

//       const tx = await liquidityPoolProgram.methods
//         .swapSolToTokens(swapAmount)
//         .accounts({
//           user: wallet.publicKey,
//           userTokenAccount: userNDollarAccount,
//           poolAccount: nDollarPoolAccount,
//           solAccount: nDollarSolAccount,
//           poolAuthority: poolAuthorityPda,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           systemProgram: anchor.web3.SystemProgram.programId,
//         })
//         .rpc();

//       console.log("Bought N-Dollars tx:", tx);

//       // Логируем состояние после покупки
//       await logUserBalance("User state after buying N-Dollars");
//       await logPoolState("Pool state after buying N-Dollars");

//       // Проверяем баланс пользователя
//       const userBalance = await provider.connection.getTokenAccountBalance(
//         userNDollarAccount
//       );
//       console.log("User N-Dollar balance:", userBalance.value.uiAmountString);
//       assert(
//         Number(userBalance.value.amount) > 0,
//         "Should have N-Dollars after swap"
//       );
//     } catch (error) {
//       console.error("Error buying N-Dollars:", error);
//       throw error;
//     }
//   });

//   it("Creates new token using N-Dollars", async () => {
//     try {
//       // Создаем ATA для нового токена
//       const createAtaIx = createAssociatedTokenAccountInstruction(
//         wallet.publicKey,
//         userNewTokenAccount,
//         wallet.publicKey,
//         newTokenMint.publicKey
//       );

//       try {
//         await provider.sendAndConfirm(new Transaction().add(createAtaIx));
//         console.log(
//           "Created user new token ATA:",
//           userNewTokenAccount.toString()
//         );
//       } catch (e) {
//         console.log("ATA already exists");
//       }

//       // Логируем состояние перед созданием токена
//       await logUserBalance("User state before token creation");
//       await logPoolState("Pool state before token creation");

//       const tx = await tokenCreatorProgram.methods
//         .createUserToken(
//           "Test Token",
//           "TEST",
//           "https://test.com/token.json",
//           TOKEN_DECIMALS,
//           new BN(1_000_000_000).mul(new BN(10).pow(new BN(TOKEN_DECIMALS)))
//         )
//         .accounts({
//           mint: newTokenMint.publicKey,
//           metadata: newTokenMetadataPda,
//           tokenInfo: newTokenInfo,
//           authority: wallet.publicKey,
//           tokenAccount: userNewTokenAccount,
//           nDollarMint: nDollarMint.publicKey,
//           userNDollarAccount: userNDollarAccount,
//           poolNDollarAccount: nDollarPoolAccount,
//           poolSolAccount: nDollarSolAccount,
//           liquidityPoolProgram: liquidityPoolProgram.programId,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//           systemProgram: anchor.web3.SystemProgram.programId,
//           rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//           tokenMetadataProgram: new PublicKey(
//             "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
//           ),
//         })
//         .signers([newTokenMint])
//         .rpc();

//       console.log("Created new token tx:", tx);

//       // Проверяем баланс нового токена
//       const newTokenBalance = await provider.connection.getTokenAccountBalance(
//         userNewTokenAccount
//       );
//       console.log("New token balance:", newTokenBalance.value.uiAmountString);

//       // Проверяем, что N-доллары были списаны
//       const nDollarBalance = await provider.connection.getTokenAccountBalance(
//         userNDollarAccount
//       );
//       console.log(
//         "Remaining N-Dollar balance:",
//         nDollarBalance.value.uiAmountString
//       );

//       await logPoolState("Final pool state");
//     } catch (error) {
//       console.error("Error creating new token:", error);
//       throw error;
//     }
//   });
// });
