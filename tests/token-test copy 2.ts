// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import { PublicKey, Keypair } from "@solana/web3.js";
// import {
//   TOKEN_PROGRAM_ID,
//   getAssociatedTokenAddress,
//   ASSOCIATED_TOKEN_PROGRAM_ID,
//   createAssociatedTokenAccountInstruction,
//   createMint,
// } from "@solana/spl-token";
// import { assert } from "chai";
// import { BN } from "bn.js";
// import { Transaction } from "@solana/web3.js";

// const TOKEN_DECIMALS = 9;

// describe("Simplified Token and Liquidity Pool Test", () => {
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//   const program = anchor.workspace.NDollar as Program;
//   const liquidityPoolProgram = anchor.workspace.LiquidityPool as Program;
//   const wallet = provider.wallet as anchor.Wallet;

//   // Объявляем переменные без начальных значений
//   let mint: Keypair;
//   let metadataPda: PublicKey;
//   let poolTokenAccount: PublicKey;
//   let solAccountPda: PublicKey;
//   let userTokenAccount: PublicKey;
//   let poolAuthorityPda: PublicKey;
//   let mintSchedulePda: PublicKey;

//   // Добавляем функцию для аирдропа
//   async function airdropSol(address: PublicKey, amount: number) {
//     const signature = await provider.connection.requestAirdrop(address, amount);
//     await provider.connection.confirmTransaction(signature);
//     console.log(
//       `Airdropped ${
//         amount / anchor.web3.LAMPORTS_PER_SOL
//       } SOL to ${address.toString()}`
//     );
//   }

//   // Добавим функцию для вывода состояния пула
//   async function logPoolState(message: string) {
//     const poolTokenBalance = await provider.connection.getTokenAccountBalance(
//       poolTokenAccount
//     );
//     const solBalance = await provider.connection.getBalance(solAccountPda);
//     const price = await liquidityPoolProgram.methods
//       .getPrice()
//       .accounts({
//         poolAccount: poolTokenAccount,
//         solAccount: solAccountPda,
//       })
//       .view();

//     console.log("\n=== " + message + " ===");
//     console.log(
//       "Pool tokens:",
//       Number(poolTokenBalance.value.uiAmountString).toLocaleString()
//     );
//     console.log(
//       "Pool SOL:",
//       (solBalance / anchor.web3.LAMPORTS_PER_SOL).toLocaleString(),
//       "SOL"
//     );
//     // Форматируем цену для лучшей читаемости (исправленная версия)
//     const priceStr = new BN(price.toString()).toString();
//     console.log("Token price:", priceStr, "tokens per LAMPORT");
//     // Также покажем цену за 1 SOL для удобства
//     const pricePerSol = Number(priceStr) * 1_000_000_000;
//     console.log("Token price:", pricePerSol.toLocaleString(), "tokens per SOL");
//     console.log("===============\n");
//   }

//   // Добавим функцию для вывода баланса пользователя
//   async function logUserBalance(message: string) {
//     const userSolBalance = await provider.connection.getBalance(
//       wallet.publicKey
//     );
//     let userTokenBalance;
//     try {
//       userTokenBalance = await provider.connection.getTokenAccountBalance(
//         userTokenAccount
//       );
//     } catch {
//       userTokenBalance = { value: { uiAmountString: "0" } };
//     }

//     console.log("\n=== " + message + " ===");
//     console.log(
//       "User SOL:",
//       (userSolBalance / anchor.web3.LAMPORTS_PER_SOL).toLocaleString(),
//       "SOL"
//     );
//     console.log(
//       "User tokens:",
//       Number(userTokenBalance.value.uiAmountString).toLocaleString()
//     );
//     console.log("===============\n");
//   }

//   before(async () => {
//     // Инициализируем все PDA и аккаунты перед тестами
//     mint = Keypair.generate();

//     const metadataProgramId = new PublicKey(
//       "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
//     );

//     [metadataPda] = PublicKey.findProgramAddressSync(
//       [
//         Buffer.from("metadata"),
//         metadataProgramId.toBuffer(),
//         mint.publicKey.toBuffer(),
//       ],
//       metadataProgramId
//     );

//     [poolAuthorityPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("pool_authority")],
//       liquidityPoolProgram.programId
//     );

//     [solAccountPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("sol_account"), mint.publicKey.toBuffer()],
//       liquidityPoolProgram.programId
//     );

//     poolTokenAccount = await getAssociatedTokenAddress(
//       mint.publicKey,
//       poolAuthorityPda,
//       true
//     );

//     userTokenAccount = await getAssociatedTokenAddress(
//       mint.publicKey,
//       wallet.publicKey
//     );

//     [mintSchedulePda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("mint_schedule"), mint.publicKey.toBuffer()],
//       program.programId
//     );

//     // Добавляем аирдроп для пользователя
//     const userBalance = await provider.connection.getBalance(wallet.publicKey);
//     if (userBalance < anchor.web3.LAMPORTS_PER_SOL) {
//       await airdropSol(wallet.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL); // 2 SOL для тестов
//     }
//     console.log(
//       "User SOL balance:",
//       (await provider.connection.getBalance(wallet.publicKey)) /
//         anchor.web3.LAMPORTS_PER_SOL,
//       "SOL"
//     );
//   });

//   it("Creates token and initializes liquidity pool via CPI", async () => {
//     const tx = await program.methods
//       .createToken(
//         "One-Click Token",
//         "ONE",
//         "https://oneclick.com/token.json",
//         TOKEN_DECIMALS
//       )
//       .accounts({
//         mint: mint.publicKey,
//         metadata: metadataPda,
//         mintSchedule: mintSchedulePda,
//         authority: wallet.publicKey,
//         poolAccount: poolTokenAccount,
//         solAccount: solAccountPda,
//         poolAuthority: poolAuthorityPda,
//         liquidityPoolProgram: liquidityPoolProgram.programId,
//         tokenProgram: TOKEN_PROGRAM_ID,
//         associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//         systemProgram: anchor.web3.SystemProgram.programId,
//         rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//         tokenMetadataProgram: new PublicKey(
//           "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
//         ),
//       })
//       .signers([mint])
//       .rpc();

//     console.log("Create token tx:", tx);
//     await logPoolState("Initial pool state after creation");

//     // Проверяем балансы после создания
//     const poolTokenBalance = await provider.connection.getTokenAccountBalance(
//       poolTokenAccount
//     );

//     const solBalance = await provider.connection.getBalance(solAccountPda);

//     assert.strictEqual(
//       poolTokenBalance.value.uiAmount,
//       108_000_000,
//       "Pool should have initial minted tokens"
//     );

//     assert.approximately(
//       solBalance,
//       500_000_000,
//       1_000_000,
//       "Pool should have approximately 0.5 SOL initial balance (excluding rent)"
//     );
//   });

//   it("Allows swapping SOL for tokens and back", async () => {
//     // Получаем bump для pool_authority
//     const [poolAuthorityPda, poolAuthorityBump] =
//       PublicKey.findProgramAddressSync(
//         [Buffer.from("pool_authority")],
//         liquidityPoolProgram.programId
//       );

//     // Создаем ATA для пользователя, если его еще нет
//     const createAtaIx = createAssociatedTokenAccountInstruction(
//       wallet.publicKey, // payer
//       userTokenAccount, // ata
//       wallet.publicKey, // owner
//       mint.publicKey // mint
//     );

//     try {
//       await provider.sendAndConfirm(new Transaction().add(createAtaIx));
//       console.log("Created user ATA:", userTokenAccount.toString());
//     } catch (e) {
//       console.log("ATA already exists");
//     }

//     // Логируем начальное состояние
//     await logUserBalance("Initial user state");
//     await logPoolState("Initial pool state");

//     // Получаем начальную цену
//     const initialPrice = await liquidityPoolProgram.methods
//       .getPrice()
//       .accounts({
//         poolAccount: poolTokenAccount,
//         solAccount: solAccountPda,
//       })
//       .view();

//     console.log("Initial price:", initialPrice.toString());

//     // Покупаем токены за 0.1 SOL
//     const swapAmount = new BN(100_000_000); // 0.1 SOL
//     console.log(
//       "\nSwapping",
//       swapAmount.toNumber() / 1_000_000_000,
//       "SOL to tokens"
//     );

//     const swapTx = await liquidityPoolProgram.methods
//       .swapSolToTokens(swapAmount)
//       .accounts({
//         user: wallet.publicKey,
//         userTokenAccount: userTokenAccount,
//         poolAccount: poolTokenAccount,
//         solAccount: solAccountPda,
//         poolAuthority: poolAuthorityPda,
//         tokenProgram: TOKEN_PROGRAM_ID,
//         systemProgram: anchor.web3.SystemProgram.programId,
//       })
//       .signers([wallet.payer])
//       .preInstructions([
//         anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
//           units: 300_000,
//         }),
//       ])
//       .rpc();

//     console.log("Swap SOL to tokens tx:", swapTx);

//     // Логируем состояние после покупки
//     await logUserBalance("User state after buying tokens");
//     await logPoolState("Pool state after buying tokens");

//     // Получаем баланс для расчета продажи
//     const userTokenBalance = await provider.connection.getTokenAccountBalance(
//       userTokenAccount
//     );

//     // Продаем половину полученных токенов обратно
//     const sellAmount = new BN(userTokenBalance.value.amount).divn(2);
//     console.log(
//       "\nSelling",
//       (Number(sellAmount.toString()) / 1_000_000_000).toLocaleString(),
//       "tokens back to pool"
//     );

//     const sellTx = await liquidityPoolProgram.methods
//       .swapTokensToSol(sellAmount)
//       .accounts({
//         user: wallet.publicKey,
//         userTokenAccount: userTokenAccount,
//         poolAccount: poolTokenAccount,
//         solAccount: solAccountPda,
//         mint: mint.publicKey,
//         poolAuthority: poolAuthorityPda,
//         tokenProgram: TOKEN_PROGRAM_ID,
//       })
//       .rpc();

//     console.log("Swap tokens to SOL tx:", sellTx);

//     // Логируем финальное состояние
//     await logUserBalance("Final user state");
//     await logPoolState("Final pool state");

//     // Получаем финальные балансы для проверок
//     const finalUserTokenBalance =
//       await provider.connection.getTokenAccountBalance(userTokenAccount);
//     const finalPoolTokenBalance =
//       await provider.connection.getTokenAccountBalance(poolTokenAccount);
//     const finalSolBalance = await provider.connection.getBalance(solAccountPda);

//     // Исправленные проверки
//     assert(
//       Number(finalUserTokenBalance.value.amount) <
//         Number(userTokenBalance.value.amount),
//       "User tokens should decrease after selling"
//     );
//     assert(
//       Number(finalPoolTokenBalance.value.amount) > 108_000_000,
//       "Pool tokens should increase after user sells tokens back"
//     );
//     assert(
//       finalSolBalance < 600_000_000, // меньше 0.6 SOL
//       "Pool SOL should decrease after sending SOL to user"
//     );
//   });
// });

// // // Исправления в Genesis Token Creator Test
// // describe("Genesis Token Creator Test", () => {
// //   const provider = anchor.AnchorProvider.env();
// //   anchor.setProvider(provider);

// //   const program = anchor.workspace.TokenCreator as Program;
// //   const wallet = provider.wallet as anchor.Wallet;

// //   // Переменные для хранения ключей и адресов
// //   let mint: Keypair;
// //   let defaultMintKeypair: Keypair;
// //   let defaultMint: PublicKey;
// //   let metadataPda: PublicKey;
// //   let tokenInfo: PublicKey;
// //   let founderAccount: PublicKey;
// //   let marketingAccount: PublicKey;
// //   let operationalAccount: PublicKey;
// //   let bondingCurveAccount: PublicKey;
// //   let bondingCurveAuthority: PublicKey;
// //   let globalConfig: PublicKey;
// //   let globalConfigBump: number;

// //   before(async () => {
// //     // Проверяем соответствие wallet.payer и wallet.publicKey
// //     console.log("Check wallet keys:");
// //     console.log("wallet.publicKey:", wallet.publicKey.toString());
// //     console.log("wallet.payer.publicKey:", wallet.payer.publicKey.toString());

// //     // Создаем дефолтный минт для глобальных аккаунтов
// //     defaultMintKeypair = Keypair.generate();
// //     defaultMint = await createMint(
// //       provider.connection,
// //       wallet.payer,
// //       wallet.publicKey,
// //       null,
// //       9,
// //       defaultMintKeypair
// //     );

// //     // Генерируем новый минт для пользовательского токена
// //     mint = Keypair.generate();

// //     // Находим PDA для метаданных
// //     const metadataProgramId = new PublicKey(
// //       "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
// //     );
// //     [metadataPda] = PublicKey.findProgramAddressSync(
// //       [
// //         Buffer.from("metadata"),
// //         metadataProgramId.toBuffer(),
// //         mint.publicKey.toBuffer(),
// //       ],
// //       metadataProgramId
// //     );

// //     // Находим PDA для token_info
// //     [tokenInfo] = PublicKey.findProgramAddressSync(
// //       [Buffer.from("token_info"), mint.publicKey.toBuffer()],
// //       program.programId
// //     );

// //     // Находим PDA для global_config с bump
// //     [globalConfig, globalConfigBump] = PublicKey.findProgramAddressSync(
// //       [Buffer.from("global_config")],
// //       program.programId
// //     );

// //     // Находим PDA для bonding_curve_authority
// //     [bondingCurveAuthority] = PublicKey.findProgramAddressSync(
// //       [Buffer.from("global_bonding_curve_authority")],
// //       program.programId
// //     );

// //     // Получаем адреса ассоциированных токен аккаунтов
// //     founderAccount = await getAssociatedTokenAddress(
// //       mint.publicKey,
// //       wallet.publicKey
// //     );

// //     // Создаем новые адреса для marketing и operational аккаунтов
// //     // Важно! Используем wallet.payer.publicKey для согласованности с сигнером
// //     marketingAccount = await getAssociatedTokenAddress(
// //       defaultMint,
// //       wallet.payer.publicKey
// //     );

// //     operationalAccount = await getAssociatedTokenAddress(
// //       defaultMint,
// //       wallet.payer.publicKey
// //     );

// //     bondingCurveAccount = await getAssociatedTokenAddress(
// //       mint.publicKey,
// //       bondingCurveAuthority,
// //       true
// //     );

// //     // Проверим, существуют ли уже аккаунты
// //     try {
// //       const marketingInfo = await provider.connection.getAccountInfo(
// //         marketingAccount
// //       );
// //       console.log("Marketing account exists:", !!marketingInfo);
// //     } catch (e) {
// //       console.log("Error checking marketing account:", e);
// //     }

// //     try {
// //       const operationalInfo = await provider.connection.getAccountInfo(
// //         operationalAccount
// //       );
// //       console.log("Operational account exists:", !!operationalInfo);
// //     } catch (e) {
// //       console.log("Error checking operational account:", e);
// //     }

// //     // Аирдроп SOL для тестов если нужно
// //     const balance = await provider.connection.getBalance(wallet.publicKey);
// //     if (balance < anchor.web3.LAMPORTS_PER_SOL) {
// //       await provider.connection.requestAirdrop(
// //         wallet.publicKey,
// //         2 * anchor.web3.LAMPORTS_PER_SOL
// //       );
// //     }
// //   });

// //   it("Initializes global accounts", async () => {
// //     try {
// //       const tx = await program.methods
// //         .initializeGlobalAccounts()
// //         .accounts({
// //           admin: wallet.payer.publicKey, // Используем wallet.payer.publicKey как admin
// //           globalConfig: globalConfig,
// //           marketingAccount: marketingAccount,
// //           operationalAccount: operationalAccount,
// //           defaultMint: defaultMint,
// //           tokenProgram: TOKEN_PROGRAM_ID,
// //           associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
// //           systemProgram: anchor.web3.SystemProgram.programId,
// //           rent: anchor.web3.SYSVAR_RENT_PUBKEY,
// //         })
// //         .signers([wallet.payer])
// //         .rpc();

// //       console.log("Initialize global accounts tx:", tx);

// //       // Проверяем, что аккаунты созданы
// //       const globalConfigAccount = await (
// //         program.account as any
// //       ).globalConfig.fetch(globalConfig);
// //       console.log("Global config initialized:", globalConfigAccount);
// //     } catch (error) {
// //       console.error("Error initializing global accounts:", error);
// //       throw error;
// //     }
// //   });

// //   it("Creates a new user token", async () => {
// //     try {
// //       const totalSupply = new BN(1_000_000_000).mul(
// //         new BN(10).pow(new BN(TOKEN_DECIMALS))
// //       ); // 1 миллиард токенов с учетом decimals

// //       // Получаем глобальный конфиг для получения admin
// //       const globalConfigAccount = await (
// //         program.account as any
// //       ).globalConfig.fetch(globalConfig);
// //       console.log("Global config:", globalConfigAccount);
// //       const globalAdmin = globalConfigAccount.admin;
// //       console.log("Global admin pubkey:", globalAdmin.toString());

// //       // Переоткрываем founder_account, чтобы он был связан с новым минтом
// //       founderAccount = await getAssociatedTokenAddress(
// //         mint.publicKey,
// //         wallet.payer.publicKey // Используем wallet.payer.publicKey
// //       );

// //       // Создаем АТА для маркетингового аккаунта с новым токеном
// //       const tokenMarketingAccount = await getAssociatedTokenAddress(
// //         mint.publicKey,
// //         globalAdmin, // Используем globalAdmin как владельца
// //         true // allowOwnerOffCurve = true для PDA, если globalAdmin это PDA
// //       );

// //       // Создаем АТА для операционного аккаунта с новым токеном
// //       const tokenOperationalAccount = await getAssociatedTokenAddress(
// //         mint.publicKey,
// //         globalAdmin, // Используем globalAdmin как владельца
// //         true // allowOwnerOffCurve = true для PDA, если globalAdmin это PDA
// //       );

// //       // Получаем адрес бондингового АТА
// //       const bondingCurveAccount = await getAssociatedTokenAddress(
// //         mint.publicKey,
// //         bondingCurveAuthority,
// //         true // allowOwnerOffCurve = true для PDA
// //       );

// //       console.log("Founder account:", founderAccount.toString());
// //       console.log("Token marketing account:", tokenMarketingAccount.toString());
// //       console.log(
// //         "Token operational account:",
// //         tokenOperationalAccount.toString()
// //       );
// //       console.log("Bonding curve account:", bondingCurveAccount.toString());

// //       const tx = await program.methods
// //         .createUserToken(
// //           "Test Token",
// //           "TEST",
// //           "https://test.com/token.json",
// //           9,
// //           totalSupply
// //         )
// //         .accounts({
// //           globalConfig: globalConfig,
// //           mint: mint.publicKey,
// //           metadata: metadataPda,
// //           tokenInfo: tokenInfo,
// //           authority: wallet.payer.publicKey,
// //           founderAccount: founderAccount,
// //           marketingAccount: marketingAccount,
// //           operationalAccount: operationalAccount,
// //           tokenMarketingAccount: tokenMarketingAccount,
// //           tokenOperationalAccount: tokenOperationalAccount,
// //           globalAdmin: globalAdmin,
// //           bondingCurveAccount: bondingCurveAccount,
// //           bondingCurveAuthority: bondingCurveAuthority,
// //           tokenProgram: TOKEN_PROGRAM_ID,
// //           associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
// //           systemProgram: anchor.web3.SystemProgram.programId,
// //           rent: anchor.web3.SYSVAR_RENT_PUBKEY,
// //           tokenMetadataProgram: new PublicKey(
// //             "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
// //           ),
// //         })
// //         .preInstructions([
// //           anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
// //             units: 400_000,
// //           }),
// //         ])
// //         .signers([mint, wallet.payer])
// //         .rpc();

// //       console.log("Create token tx:", tx);

// //       // Проверяем балансы созданных аккаунтов
// //       try {
// //         const founderBalance = await provider.connection.getTokenAccountBalance(
// //           founderAccount
// //         );
// //         console.log("Founder balance:", founderBalance.value.uiAmountString);
// //       } catch (e) {
// //         console.error("Error checking founder balance:", e);
// //       }

// //       try {
// //         const marketingBalance =
// //           await provider.connection.getTokenAccountBalance(
// //             tokenMarketingAccount
// //           );
// //         console.log(
// //           "Marketing token balance:",
// //           marketingBalance.value.uiAmountString
// //         );
// //       } catch (e) {
// //         console.error("Error checking marketing balance:", e);
// //       }

// //       try {
// //         const operationalBalance =
// //           await provider.connection.getTokenAccountBalance(
// //             tokenOperationalAccount
// //           );
// //         console.log(
// //           "Operational token balance:",
// //           operationalBalance.value.uiAmountString
// //         );
// //       } catch (e) {
// //         console.error("Error checking operational balance:", e);
// //       }

// //       try {
// //         const bondingCurveBalance =
// //           await provider.connection.getTokenAccountBalance(bondingCurveAccount);
// //         console.log(
// //           "Bonding curve balance:",
// //           bondingCurveBalance.value.uiAmountString
// //         );
// //       } catch (e) {
// //         console.error("Error checking bonding curve balance:", e);
// //       }

// //       const tokenInfoAccount = await (program.account as any).tokenInfo.fetch(
// //         tokenInfo
// //       );
// //       console.log("Token info:", {
// //         mint: tokenInfoAccount.mint.toString(),
// //         authority: tokenInfoAccount.authority.toString(),
// //         totalSupply: new BN(tokenInfoAccount.totalSupply.toString())
// //           .div(new BN(10).pow(new BN(TOKEN_DECIMALS)))
// //           .toString(),
// //       });
// //     } catch (error) {
// //       console.error("Error creating user token:", error);
// //       throw error;
// //     }
// //   });
// // });

// // Исправленный код в token-test.ts для правильного распределения токенов
// // Внесём только необходимые изменения в describe("Genesis Token Creator Test", () => {...})

// describe("Genesis Token Creator Test", () => {
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//   const program = anchor.workspace.TokenCreator as Program;
//   const wallet = provider.wallet as anchor.Wallet;

//   // Переменные для хранения ключей и адресов
//   let mint: Keypair;
//   let defaultMintKeypair: Keypair;
//   let defaultMint: PublicKey;
//   let metadataPda: PublicKey;
//   let tokenInfo: PublicKey;
//   let founderAccount: PublicKey;
//   let marketingAccount: PublicKey;
//   let operationalAccount: PublicKey;
//   let bondingCurveAccount: PublicKey;
//   let bondingCurveAuthority: PublicKey;
//   let globalConfig: PublicKey;
//   let globalConfigBump: number;

//   // Создаём дополнительные ключевые пары для маркетингового и операционного адресов
//   const marketingKeypair = Keypair.generate();
//   const operationalKeypair = Keypair.generate();

//   before(async () => {
//     // Проверяем соответствие wallet.payer и wallet.publicKey
//     console.log("Check wallet keys:");
//     console.log("wallet.publicKey:", wallet.publicKey.toString());
//     console.log("wallet.payer.publicKey:", wallet.payer.publicKey.toString());
//     console.log("Marketing keypair:", marketingKeypair.publicKey.toString());
//     console.log(
//       "Operational keypair:",
//       operationalKeypair.publicKey.toString()
//     );

//     // Создаем дефолтный минт для глобальных аккаунтов
//     defaultMintKeypair = Keypair.generate();
//     defaultMint = await createMint(
//       provider.connection,
//       wallet.payer,
//       wallet.publicKey,
//       null,
//       9,
//       defaultMintKeypair
//     );

//     // Генерируем новый минт для пользовательского токена
//     mint = Keypair.generate();

//     // Находим PDA для метаданных
//     const metadataProgramId = new PublicKey(
//       "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
//     );
//     [metadataPda] = PublicKey.findProgramAddressSync(
//       [
//         Buffer.from("metadata"),
//         metadataProgramId.toBuffer(),
//         mint.publicKey.toBuffer(),
//       ],
//       metadataProgramId
//     );

//     // Находим PDA для token_info
//     [tokenInfo] = PublicKey.findProgramAddressSync(
//       [Buffer.from("token_info"), mint.publicKey.toBuffer()],
//       program.programId
//     );

//     // Находим PDA для global_config с bump
//     [globalConfig, globalConfigBump] = PublicKey.findProgramAddressSync(
//       [Buffer.from("global_config")],
//       program.programId
//     );

//     // Находим PDA для bonding_curve_authority
//     [bondingCurveAuthority] = PublicKey.findProgramAddressSync(
//       [Buffer.from("global_bonding_curve_authority")],
//       program.programId
//     );

//     // Получаем адреса ассоциированных токен аккаунтов для founder
//     founderAccount = await getAssociatedTokenAddress(
//       mint.publicKey,
//       wallet.publicKey
//     );

//     // Создаем новые адреса для marketing и operational аккаунтов
//     // Используем wallet.payer.publicKey для согласованности с сигнером
//     marketingAccount = await getAssociatedTokenAddress(
//       defaultMint,
//       wallet.payer.publicKey
//     );

//     operationalAccount = await getAssociatedTokenAddress(
//       defaultMint,
//       wallet.payer.publicKey
//     );

//     bondingCurveAccount = await getAssociatedTokenAddress(
//       mint.publicKey,
//       bondingCurveAuthority,
//       true
//     );

//     // Аирдроп SOL для тестов если нужно
//     const balance = await provider.connection.getBalance(wallet.publicKey);
//     if (balance < anchor.web3.LAMPORTS_PER_SOL) {
//       await provider.connection.requestAirdrop(
//         wallet.publicKey,
//         2 * anchor.web3.LAMPORTS_PER_SOL
//       );
//     }

//     // Аирдроп SOL для новых аккаунтов
//     await provider.connection.requestAirdrop(
//       marketingKeypair.publicKey,
//       0.1 * anchor.web3.LAMPORTS_PER_SOL
//     );

//     await provider.connection.requestAirdrop(
//       operationalKeypair.publicKey,
//       0.1 * anchor.web3.LAMPORTS_PER_SOL
//     );
//   });

//   it("Initializes global accounts", async () => {
//     try {
//       const tx = await program.methods
//         .initializeGlobalAccounts()
//         .accounts({
//           admin: wallet.payer.publicKey, // Используем wallet.payer.publicKey как admin
//           globalConfig: globalConfig,
//           marketingAccount: marketingAccount,
//           operationalAccount: operationalAccount,
//           defaultMint: defaultMint,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//           systemProgram: anchor.web3.SystemProgram.programId,
//           rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//         })
//         .signers([wallet.payer])
//         .rpc();

//       console.log("Initialize global accounts tx:", tx);

//       // Проверяем, что аккаунты созданы
//       const globalConfigAccount = await (
//         program.account as any
//       ).globalConfig.fetch(globalConfig);
//       console.log("Global config initialized:", globalConfigAccount);
//     } catch (error) {
//       console.error("Error initializing global accounts:", error);
//       throw error;
//     }
//   });

//   it("Creates a new user token", async () => {
//     try {
//       const totalSupply = new BN(1_000_000_000).mul(
//         new BN(10).pow(new BN(TOKEN_DECIMALS))
//       ); // 1 миллиард токенов с учетом decimals

//       // Получаем глобальный конфиг для получения admin
//       const globalConfigAccount = await (
//         program.account as any
//       ).globalConfig.fetch(globalConfig);

//       const globalAdmin = globalConfigAccount.admin;
//       console.log("Global admin pubkey:", globalAdmin.toString());

//       // Переоткрываем founder_account, чтобы он был связан с новым минтом
//       founderAccount = await getAssociatedTokenAddress(
//         mint.publicKey,
//         wallet.payer.publicKey
//       );

//       // Создаем АТА для маркетингового аккаунта с новым токеном
//       // ИЗМЕНЕНИЕ: Используем отдельный keypair для маркетингового аккаунта
//       const tokenMarketingAccount = await getAssociatedTokenAddress(
//         mint.publicKey,
//         marketingKeypair.publicKey
//       );

//       // Создаем АТА для операционного аккаунта с новым токеном
//       // ИЗМЕНЕНИЕ: Используем отдельный keypair для операционного аккаунта
//       const tokenOperationalAccount = await getAssociatedTokenAddress(
//         mint.publicKey,
//         operationalKeypair.publicKey
//       );

//       // Получаем адрес бондингового АТА
//       const bondingCurveAccount = await getAssociatedTokenAddress(
//         mint.publicKey,
//         bondingCurveAuthority,
//         true // allowOwnerOffCurve = true для PDA
//       );

//       console.log("Founder account:", founderAccount.toString());
//       console.log("Token marketing account:", tokenMarketingAccount.toString());
//       console.log(
//         "Token operational account:",
//         tokenOperationalAccount.toString()
//       );
//       console.log("Bonding curve account:", bondingCurveAccount.toString());

//       // Создаем АТА для маркетингового и операционного аккаунтов заранее
//       try {
//         const createMarketingAtaIx = createAssociatedTokenAccountInstruction(
//           wallet.payer.publicKey,
//           tokenMarketingAccount,
//           marketingKeypair.publicKey,
//           mint.publicKey
//         );

//         const createOperationalAtaIx = createAssociatedTokenAccountInstruction(
//           wallet.payer.publicKey,
//           tokenOperationalAccount,
//           operationalKeypair.publicKey,
//           mint.publicKey
//         );

//         // Отправляем транзакцию на создание АТА
//         await provider.sendAndConfirm(
//           new Transaction()
//             .add(createMarketingAtaIx)
//             .add(createOperationalAtaIx),
//           [wallet.payer]
//         );

//         console.log("ATAs created for marketing and operational accounts");
//       } catch (e) {
//         console.log("Error creating ATAs, they may already exist:", e);
//       }
//       // const tempAuthKeypair = Keypair.generate();

//       // Дополнительные изменения в token-test.ts для работы с обновленным контрактом

//       // В тесте создания токена нужно добавить новые аккаунты
//       const tx = await program.methods
//         .createUserToken(
//           "Test Token",
//           "TEST",
//           "https://test.com/token.json",
//           9,
//           totalSupply
//         )
//         .accounts({
//           globalConfig: globalConfig,
//           mint: mint.publicKey,
//           metadata: metadataPda,
//           tokenInfo: tokenInfo,
//           authority: wallet.payer.publicKey,
//           founderAccount: founderAccount,
//           marketingAccount: marketingAccount,
//           operationalAccount: operationalAccount,
//           tokenMarketingAccount: tokenMarketingAccount,
//           tokenMarketingOwner: marketingKeypair.publicKey, // Добавляем владельца маркетингового аккаунта
//           tokenOperationalAccount: tokenOperationalAccount,
//           tokenOperationalOwner: operationalKeypair.publicKey, // Добавляем владельца операционного аккаунта
//           globalAdmin: globalAdmin,
//           bondingCurveAccount: bondingCurveAccount,
//           bondingCurveAuthority: bondingCurveAuthority,
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
//         .signers([mint, wallet.payer])
//         .rpc();

//       console.log("Create token tx:", tx);

//       // Проверяем балансы созданных аккаунтов
//       try {
//         const founderBalance = await provider.connection.getTokenAccountBalance(
//           founderAccount
//         );
//         console.log("Founder balance:", founderBalance.value.uiAmountString);
//       } catch (e) {
//         console.error("Error checking founder balance:", e);
//       }

//       try {
//         const marketingBalance =
//           await provider.connection.getTokenAccountBalance(
//             tokenMarketingAccount
//           );
//         console.log(
//           "Marketing token balance:",
//           marketingBalance.value.uiAmountString
//         );
//       } catch (e) {
//         console.error("Error checking marketing balance:", e);
//       }

//       try {
//         const operationalBalance =
//           await provider.connection.getTokenAccountBalance(
//             tokenOperationalAccount
//           );
//         console.log(
//           "Operational token balance:",
//           operationalBalance.value.uiAmountString
//         );
//       } catch (e) {
//         console.error("Error checking operational balance:", e);
//       }

//       try {
//         const bondingCurveBalance =
//           await provider.connection.getTokenAccountBalance(bondingCurveAccount);
//         console.log(
//           "Bonding curve balance:",
//           bondingCurveBalance.value.uiAmountString
//         );
//       } catch (e) {
//         console.error("Error checking bonding curve balance:", e);
//       }

//       const tokenInfoAccount = await (program.account as any).tokenInfo.fetch(
//         tokenInfo
//       );
//       console.log("Token info:", {
//         mint: tokenInfoAccount.mint.toString(),
//         authority: tokenInfoAccount.authority.toString(),
//         totalSupply: new BN(tokenInfoAccount.totalSupply.toString())
//           .div(new BN(10).pow(new BN(TOKEN_DECIMALS)))
//           .toString(),
//       });
//     } catch (error) {
//       console.error("Error creating user token:", error);
//       throw error;
//     }
//   });
// });
