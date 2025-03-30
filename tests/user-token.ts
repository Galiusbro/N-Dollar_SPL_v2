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
//   const wallet = provider.wallet as anchor.Wallet;

//   // Ключи и адреса для N-доллара
//   let nDollarMint: Keypair;
//   let nDollarMetadataPda: PublicKey;
//   let nDollarPoolAccount: PublicKey;
//   let nDollarSolAccount: PublicKey;
//   let userNDollarAccount: PublicKey;
//   let poolAuthorityPda: PublicKey;
//   let mintSchedulePda: PublicKey;
//   let liquidityPoolStatePda: PublicKey;

//   // Ключи и адреса для создаваемого токена
//   let newTokenMint: Keypair;
//   let newTokenMetadataPda: PublicKey;
//   let newTokenInfo: PublicKey;
//   let userNewTokenAccount: PublicKey;

//   const TOKEN_DECIMALS = 9;

//   // Добавляем функцию для вывода состояния пула
//   async function logPoolState(message: string) {
//     try {
//       const poolTokenBalance = await provider.connection.getTokenAccountBalance(
//         nDollarPoolAccount
//       );
//       const solBalance = await provider.connection.getBalance(
//         nDollarSolAccount
//       );

//       console.log("\n=== " + message + " ===");
//       console.log(
//         "Pool N-Dollars:",
//         Number(poolTokenBalance.value.uiAmountString).toLocaleString()
//       );
//       console.log(
//         "Pool SOL:",
//         (solBalance / anchor.web3.LAMPORTS_PER_SOL).toLocaleString(),
//         "SOL"
//       );

//       try {
//         const price = await nDollarProgram.methods
//           .getPrice()
//           .accounts({
//             pool_account: nDollarPoolAccount,
//             sol_account: nDollarSolAccount,
//           })
//           .rpc();

//         console.log("Token price:", price.toString(), "tokens per LAMPORT");
//         const pricePerSol = Number(price.toString()) * 1_000_000_000;
//         console.log(
//           "Token price:",
//           pricePerSol.toLocaleString(),
//           "tokens per SOL"
//         );
//       } catch (e) {
//         console.log("Could not fetch price:", e);
//       }

//       console.log("===============\n");
//     } catch (error) {
//       console.error("Error logging pool state:", error);
//     }
//   }

//   // Добавляем функцию для вывода баланса пользователя
//   async function logUserBalance(message: string) {
//     try {
//       const userSolBalance = await provider.connection.getBalance(
//         wallet.publicKey
//       );
//       let userNDollarBalance;
//       try {
//         userNDollarBalance = await provider.connection.getTokenAccountBalance(
//           userNDollarAccount
//         );
//       } catch {
//         userNDollarBalance = { value: { uiAmountString: "0" } };
//       }

//       console.log("\n=== " + message + " ===");
//       console.log(
//         "User SOL:",
//         (userSolBalance / anchor.web3.LAMPORTS_PER_SOL).toLocaleString(),
//         "SOL"
//       );
//       console.log(
//         "User N-Dollars:",
//         Number(userNDollarBalance.value.uiAmountString).toLocaleString()
//       );
//       console.log("===============\n");
//     } catch (error) {
//       console.error("Error logging user balance:", error);
//     }
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

//     // Находим PDA для состояния пула ликвидности
//     [liquidityPoolStatePda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("liquidity_pool"), nDollarMint.publicKey.toBuffer()],
//       nDollarProgram.programId
//     );

//     // Находим PDA для pool_authority
//     [poolAuthorityPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("pool_authority")],
//       nDollarProgram.programId
//     );

//     // Находим PDA для sol_account
//     [nDollarSolAccount] = PublicKey.findProgramAddressSync(
//       [Buffer.from("sol_account"), nDollarMint.publicKey.toBuffer()],
//       nDollarProgram.programId
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

//     // Аирдропим 2 SOL для инициализации N-доллара и пула
//     const signature = await provider.connection.requestAirdrop(
//       wallet.publicKey,
//       2 * anchor.web3.LAMPORTS_PER_SOL
//     );
//     await provider.connection.confirmTransaction(signature);

//     console.log(
//       "Initial user SOL balance:",
//       (await provider.connection.getBalance(wallet.publicKey)) /
//         anchor.web3.LAMPORTS_PER_SOL,
//       "SOL"
//     );
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
//           liquidityPoolState: liquidityPoolStatePda,
//           authority: wallet.publicKey,
//           poolAuthority: poolAuthorityPda,
//           poolTokenAccount: nDollarPoolAccount,
//           solAccount: nDollarSolAccount,
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

//       // После создания N-доллара и пула, устанавливаем точный баланс SOL для пользователя
//       const currentBalance = await provider.connection.getBalance(
//         wallet.publicKey
//       );

//       // Отправляем весь лишний баланс на временный аккаунт
//       const tempKeypair = Keypair.generate();
//       const transferTx = new Transaction().add(
//         anchor.web3.SystemProgram.transfer({
//           fromPubkey: wallet.publicKey,
//           toPubkey: tempKeypair.publicKey,
//           lamports: currentBalance - 200_000_000, // Оставляем 0.2 SOL
//         })
//       );
//       await provider.sendAndConfirm(transferTx);

//       console.log(
//         "Set user SOL balance to:",
//         (await provider.connection.getBalance(wallet.publicKey)) /
//           anchor.web3.LAMPORTS_PER_SOL,
//         "SOL"
//       );
//     } catch (error) {
//       console.error("Error creating N-Dollar:", error);
//       throw error;
//     }
//   });

//   it("Swaps SOL for N-Dollars", async () => {
//     try {
//       // Инициализируем пул ликвидности, если он еще не инициализирован
//       try {
//         // Создаем PDA для pool_authority
//         const [poolAuthority] = PublicKey.findProgramAddressSync(
//           [Buffer.from("pool_authority")],
//           nDollarProgram.programId
//         );

//         // Создаем PDA для sol_account
//         const [solAccount] = PublicKey.findProgramAddressSync(
//           [Buffer.from("sol_account"), nDollarMint.publicKey.toBuffer()],
//           nDollarProgram.programId
//         );

//         // Создаем ATA для пула
//         const poolTokenAccount = await getAssociatedTokenAddress(
//           nDollarMint.publicKey,
//           poolAuthority,
//           true
//         );

//         await nDollarProgram.methods
//           .initializeNDollarAccount()
//           .accounts({
//             poolAccount: poolTokenAccount,
//             poolAuthority: poolAuthority,
//             mint: nDollarMint.publicKey,
//             initializer: wallet.publicKey,
//             tokenProgram: TOKEN_PROGRAM_ID,
//             associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//             systemProgram: anchor.web3.SystemProgram.programId,
//             rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//           })
//           .rpc();

//         // Инициализируем SOL аккаунт
//         await nDollarProgram.methods
//           .initializeSolAccount()
//           .accounts({
//             solAccount: solAccount,
//             poolAuthority: poolAuthority,
//             mint: nDollarMint.publicKey,
//             initializer: wallet.publicKey,
//             systemProgram: anchor.web3.SystemProgram.programId,
//           })
//           .rpc();

//         console.log("Initialized liquidity pool and SOL account");
//       } catch (e) {
//         console.log("Liquidity pool already initialized");
//       }

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
//       await logUserBalance("Initial user state before buying N-Dollars");
//       await logPoolState("Initial pool state before buying N-Dollars");

//       // Получаем текущий баланс SOL пользователя
//       const userSolBalance = await provider.connection.getBalance(
//         wallet.publicKey
//       );
//       // Оставляем немного SOL на комиссию транзакции
//       const swapAmount = userSolBalance - 5000; // Оставляем 5000 лампортов на комиссию
//       console.log(
//         "\nSwapping all SOL:",
//         swapAmount / anchor.web3.LAMPORTS_PER_SOL,
//         "SOL to N-Dollars"
//       );

//       // Создаем PDA для pool_authority
//       const [poolAuthority] = PublicKey.findProgramAddressSync(
//         [Buffer.from("pool_authority")],
//         nDollarProgram.programId
//       );

//       // Создаем PDA для sol_account
//       const [solAccount] = PublicKey.findProgramAddressSync(
//         [Buffer.from("sol_account"), nDollarMint.publicKey.toBuffer()],
//         nDollarProgram.programId
//       );

//       // Создаем ATA для пула
//       const poolTokenAccount = await getAssociatedTokenAddress(
//         nDollarMint.publicKey,
//         poolAuthority,
//         true
//       );

//       // Используем прямой перевод SOL в пул и получаем N-доллары
//       const transferTx = new Transaction().add(
//         anchor.web3.SystemProgram.transfer({
//           fromPubkey: wallet.publicKey,
//           toPubkey: solAccount,
//           lamports: swapAmount,
//         })
//       );

//       // Отправляем транзакцию
//       const tx = await provider.sendAndConfirm(transferTx);
//       console.log("Transferred SOL to pool tx:", tx);

//       // Получаем цену
//       const price = await nDollarProgram.methods
//         .getPrice()
//         .accounts({
//           pool_account: nDollarPoolAccount,
//           sol_account: nDollarSolAccount,
//         })
//         .rpc();

//       // Рассчитываем количество N-долларов, которые мы должны получить
//       const tokensToMint = new BN(swapAmount)
//         .mul(new BN("1000000000"))
//         .div(new BN(price.toString()));

//       console.log("Tokens to mint based on price:", tokensToMint.toString());

//       // Минтим N-доллары пользователю
//       const mintAtaTx = await nDollarProgram.methods
//         .mintScheduledTokens(9) // 9 decimals
//         .accounts({
//           mintSchedule: mintSchedulePda,
//           mint: nDollarMint.publicKey,
//           poolAccount: poolTokenAccount,
//           authority: wallet.publicKey,
//           tokenProgram: TOKEN_PROGRAM_ID,
//         })
//         .rpc();

//       console.log("Minted N-Dollars tx:", mintAtaTx);

//       // Логируем состояние после покупки
//       const userSolAfter = await provider.connection.getBalance(
//         wallet.publicKey
//       );
//       const poolSolAfter = await provider.connection.getBalance(solAccount);
//       const poolTokensAfter = await provider.connection.getTokenAccountBalance(
//         poolTokenAccount
//       );
//       const userTokensAfter = await provider.connection.getTokenAccountBalance(
//         userNDollarAccount
//       );

//       console.log("\nСостояние после свапа:");
//       console.log(
//         "User SOL:",
//         userSolAfter / anchor.web3.LAMPORTS_PER_SOL,
//         "SOL"
//       );
//       console.log(
//         "Pool SOL:",
//         poolSolAfter / anchor.web3.LAMPORTS_PER_SOL,
//         "SOL"
//       );
//       console.log("Pool N-Dollars:", poolTokensAfter.value.uiAmount);
//       console.log("User N-Dollars:", userTokensAfter.value.uiAmount);

//       // Проверяем, что у пользователя почти не осталось SOL
//       assert(
//         userSolAfter < 5001,
//         `User should have almost no SOL left, but has ${
//           userSolAfter / anchor.web3.LAMPORTS_PER_SOL
//         } SOL`
//       );

//       // Проверяем, что SOL перешел в пул
//       assert(
//         poolSolAfter >= swapAmount,
//         `Pool should have received SOL, but has only ${
//           poolSolAfter / anchor.web3.LAMPORTS_PER_SOL
//         } SOL`
//       );

//       // Проверяем, что пользователь получил N-доллары
//       assert(
//         Number(userTokensAfter.value.amount) > 0,
//         "Should have N-Dollars after swap"
//       );
//     } catch (error) {
//       console.error("Error buying N-Dollars:", error);
//       throw error;
//     }
//   });

//   it("Creates new token using N-Dollars with almost no SOL in wallet", async () => {
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

//       // Проверяем, что у пользователя действительно почти нет SOL
//       const userSolBefore = await provider.connection.getBalance(
//         wallet.publicKey
//       );
//       console.log(
//         "\nUser SOL before token creation:",
//         userSolBefore / anchor.web3.LAMPORTS_PER_SOL
//       );
//       assert(
//         userSolBefore < 5001,
//         "User should have almost no SOL before creating token"
//       );

//       // Логируем состояние перед созданием токена
//       await logUserBalance("User state before token creation");
//       await logPoolState("Pool state before token creation");

//       // Запоминаем балансы до создания токена
//       const userNDollarBefore =
//         await provider.connection.getTokenAccountBalance(userNDollarAccount);
//       const poolSolBefore = await provider.connection.getBalance(
//         nDollarSolAccount
//       );
//       const poolNDollarBefore =
//         await provider.connection.getTokenAccountBalance(nDollarPoolAccount);

//       // Проверяем баланс до создания токена
//       console.log("\n=== Initial Balances ===");
//       console.log("User SOL:", userSolBefore / anchor.web3.LAMPORTS_PER_SOL);
//       console.log("User N-Dollars:", userNDollarBefore.value.uiAmountString);
//       console.log("Pool SOL:", poolSolBefore / anchor.web3.LAMPORTS_PER_SOL);
//       console.log("Pool N-Dollars:", poolNDollarBefore.value.uiAmountString);

//       // Определяем нужные константы для SOL аренды
//       const TOTAL_RENT = 1_447_680 + 2_319_840 + 1_447_680 + 2_049_280; // Сумма констант из программы

//       console.log(
//         "\nTotal rent required for token creation:",
//         TOTAL_RENT / anchor.web3.LAMPORTS_PER_SOL,
//         "SOL"
//       );

//       // Проверяем, достаточно ли SOL в пуле для создания токена
//       if (poolSolBefore < TOTAL_RENT) {
//         console.log(
//           "\n⚠️ WARNING: Insufficient SOL in pool for token creation!"
//         );
//         console.log(
//           "Required:",
//           TOTAL_RENT / anchor.web3.LAMPORTS_PER_SOL,
//           "SOL"
//         );
//         console.log(
//           "Available:",
//           poolSolBefore / anchor.web3.LAMPORTS_PER_SOL,
//           "SOL"
//         );

//         // Если в пуле недостаточно SOL, добавляем его
//         if (poolSolBefore < TOTAL_RENT) {
//           console.log("\nAdding more SOL to pool...");

//           // Сначала пополняем баланс пользователя через аирдроп
//           const signature = await provider.connection.requestAirdrop(
//             wallet.publicKey,
//             1 * anchor.web3.LAMPORTS_PER_SOL
//           );
//           await provider.connection.confirmTransaction(signature);

//           // Затем переводим SOL в пул
//           const transferTx = new Transaction().add(
//             anchor.web3.SystemProgram.transfer({
//               fromPubkey: wallet.publicKey,
//               toPubkey: nDollarSolAccount,
//               lamports: TOTAL_RENT + 10_000_000, // Добавляем SOL с запасом
//             })
//           );

//           // Отправляем транзакцию
//           await provider.sendAndConfirm(transferTx);

//           // Проверяем новый баланс пула
//           const newPoolSolBalance = await provider.connection.getBalance(
//             nDollarSolAccount
//           );
//           console.log(
//             "Updated pool SOL balance:",
//             newPoolSolBalance / anchor.web3.LAMPORTS_PER_SOL,
//             "SOL"
//           );
//         }
//       }

//       // Получаем цену и рассчитываем комиссию в N-долларах
//       const price = await nDollarProgram.methods
//         .getPrice()
//         .accounts({
//           pool_account: nDollarPoolAccount,
//           sol_account: nDollarSolAccount,
//         })
//         .rpc();

//       const tokenFee = new BN(TOTAL_RENT)
//         .mul(new BN(price.toString()))
//         .div(new BN(1_000_000_000));
//       console.log("Token fee in N-dollars:", tokenFee.toString());

//       if (new BN(userNDollarBefore.value.amount).lt(tokenFee)) {
//         console.log("\n⚠️ WARNING: Insufficient N-Dollars for token creation!");
//         console.log("Required:", tokenFee.toString());
//         console.log("Available:", userNDollarBefore.value.amount);
//       }

//       console.log("\nAttempting to create token...");

//       // Отдельная дебаг-функция - выводит логи для нашей транзакции
//       async function debugLogs(signature: string) {
//         try {
//           const txInfo = await provider.connection.getTransaction(signature, {
//             commitment: "confirmed",
//             maxSupportedTransactionVersion: 0,
//           });

//           console.log("\n=== Transaction Logs ===");
//           if (txInfo?.meta?.logMessages) {
//             txInfo.meta.logMessages.forEach((log) => console.log(log));
//           } else {
//             console.log("No logs available");
//           }
//           console.log("=======================\n");
//         } catch (e) {
//           console.error("Error fetching transaction logs:", e);
//         }
//       }

//       try {
//         const tx = await tokenCreatorProgram.methods
//           .createUserToken(
//             "Test Token",
//             "TEST",
//             "https://test.com/token.json",
//             TOKEN_DECIMALS,
//             new BN(1_000_000_000)
//           )
//           .accounts({
//             mint: newTokenMint.publicKey,
//             metadata: newTokenMetadataPda,
//             tokenInfo: newTokenInfo,
//             authority: wallet.publicKey,
//             tokenAccount: userNewTokenAccount,
//             nDollarMint: nDollarMint.publicKey,
//             userNDollarAccount: userNDollarAccount,
//             poolNDollarAccount: nDollarPoolAccount,
//             poolAuthority: poolAuthorityPda,
//             poolSolAccount: nDollarSolAccount,
//             liquidityPoolState: liquidityPoolStatePda,
//             nDollarProgram: nDollarProgram.programId,
//             tokenProgram: TOKEN_PROGRAM_ID,
//             associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//             systemProgram: anchor.web3.SystemProgram.programId,
//             rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//             tokenMetadataProgram: new PublicKey(
//               "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
//             ),
//           })
//           .signers([newTokenMint])
//           .rpc({ commitment: "confirmed" });

//         console.log("Created new token tx:", tx);
//         // Выводим логи транзакции для дебага
//         await debugLogs(tx);

//         // Получаем балансы после создания токена
//         const userSolAfter = await provider.connection.getBalance(
//           wallet.publicKey
//         );
//         const userNDollarAfter =
//           await provider.connection.getTokenAccountBalance(userNDollarAccount);
//         const poolSolAfter = await provider.connection.getBalance(
//           nDollarSolAccount
//         );
//         const poolNDollarAfter =
//           await provider.connection.getTokenAccountBalance(nDollarPoolAccount);

//         console.log("\n=== Balance Changes ===");
//         console.log(
//           "User SOL change:",
//           (userSolAfter - userSolBefore) / anchor.web3.LAMPORTS_PER_SOL,
//           "SOL"
//         );
//         console.log(
//           "User N-Dollar change:",
//           Number(userNDollarAfter.value.amount) -
//             Number(userNDollarBefore.value.amount)
//         );
//         console.log(
//           "Pool SOL change:",
//           (poolSolAfter - poolSolBefore) / anchor.web3.LAMPORTS_PER_SOL,
//           "SOL"
//         );
//         console.log(
//           "Pool N-Dollar change:",
//           Number(poolNDollarAfter.value.amount) -
//             Number(poolNDollarBefore.value.amount)
//         );

//         // Проверка изменений после обновления логики:

//         // 1. N-доллары должны были переведены из аккаунта пользователя в пул
//         assert(
//           Number(userNDollarAfter.value.amount) <
//             Number(userNDollarBefore.value.amount),
//           "User N-Dollar balance should decrease"
//         );

//         assert(
//           Number(poolNDollarAfter.value.amount) >
//             Number(poolNDollarBefore.value.amount),
//           "Pool N-Dollar balance should increase"
//         );

//         // 2. SOL должен был переведен из пула пользователю через withdraw_sol
//         //    а затем потрачен на аренду
//         assert(
//           poolSolAfter < poolSolBefore,
//           "Pool SOL should decrease to pay for rent"
//         );

//         // 3. Проверка создания нового токена - должен быть успешно минчен
//         const newTokenBalance =
//           await provider.connection.getTokenAccountBalance(userNewTokenAccount);
//         console.log(
//           "\nNew token balance:",
//           newTokenBalance.value.uiAmountString
//         );
//         assert(
//           Number(newTokenBalance.value.amount) > 0,
//           "New token should be minted to user"
//         );
//       } catch (error) {
//         console.error("Error creating new token:", error);

//         // Проверяем, какие балансы после ошибки
//         console.log("\n=== Balance After Error ===");
//         console.log(
//           "User SOL:",
//           (await provider.connection.getBalance(wallet.publicKey)) /
//             anchor.web3.LAMPORTS_PER_SOL
//         );
//         console.log(
//           "Pool SOL:",
//           (await provider.connection.getBalance(nDollarSolAccount)) /
//             anchor.web3.LAMPORTS_PER_SOL
//         );

//         // Проверяем, можем ли получить SOL отдельно через withdraw_sol
//         console.log("\nTesting withdraw_sol directly...");
//         try {
//           const testWithdrawTx = await nDollarProgram.methods
//             .withdrawSol(new BN(TOTAL_RENT))
//             .accounts({
//               liquidityPoolState: liquidityPoolStatePda,
//               mint: nDollarMint.publicKey,
//               solAccount: nDollarSolAccount,
//               recipient: wallet.publicKey,
//               systemProgram: anchor.web3.SystemProgram.programId,
//             })
//             .rpc();

//           console.log("Test withdraw_sol tx:", testWithdrawTx);
//           await debugLogs(testWithdrawTx);

//           // Проверяем баланс после прямого вызова
//           console.log(
//             "User SOL after direct withdraw:",
//             (await provider.connection.getBalance(wallet.publicKey)) /
//               anchor.web3.LAMPORTS_PER_SOL
//           );
//         } catch (withdrawError) {
//           console.error("Error testing withdraw_sol:", withdrawError);
//         }

//         throw error;
//       }

//       // Итоговое состояние
//       await logPoolState("Final pool state");
//       await logUserBalance("Final user state");
//     } catch (error) {
//       console.error("Error creating new token:", error);
//       throw error;
//     }
//   });
// });
