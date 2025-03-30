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

// // Глобальная функция для аирдропа
// async function requestAirdrop(
//   provider: anchor.AnchorProvider,
//   address: PublicKey,
//   amount: number
// ) {
//   const signature = await provider.connection.requestAirdrop(
//     address,
//     amount * anchor.web3.LAMPORTS_PER_SOL
//   );
//   await provider.connection.confirmTransaction(signature);
//   console.log(`Airdropped ${amount} SOL to ${address.toString()}`);
// }

// describe("Genesis Token Creator Test", () => {
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//   const program = anchor.workspace.TokenCreator as Program;
//   const bondingCurveProgram = anchor.workspace.BondingCurve as Program;
//   const wallet = provider.wallet as anchor.Wallet;

//   // Переменные для хранения ключей и адресов
//   let mint: Keypair;
//   let metadataPda: PublicKey;
//   let tokenInfo: PublicKey;
//   let founderAccount: PublicKey;
//   let bondingCurveAccount: PublicKey;
//   let globalConfig: PublicKey;
//   let bondingCurvePda: PublicKey;
//   let solAccountPda: PublicKey;

//   // PDA для глобальных authority
//   let marketingAuthority: PublicKey;
//   let operationalAuthority: PublicKey;
//   let bondingCurveAuthority: PublicKey;

//   // Аккаунты для токенов (для проверки)
//   let tokenMarketingAccount: PublicKey;
//   let tokenOperationalAccount: PublicKey;

//   before(async () => {
//     // Проверяем соответствие wallet.payer и wallet.publicKey
//     console.log("Check wallet keys:");
//     console.log("wallet.publicKey:", wallet.publicKey.toString());
//     console.log("wallet.payer.publicKey:", wallet.payer.publicKey.toString());

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

//     // Находим PDA для global_config
//     [globalConfig] = PublicKey.findProgramAddressSync(
//       [Buffer.from("global_config")],
//       program.programId
//     );

//     // Находим PDA для маркетингового authority
//     [marketingAuthority] = PublicKey.findProgramAddressSync(
//       [Buffer.from("global_marketing_authority")],
//       program.programId
//     );

//     // Находим PDA для операционного authority
//     [operationalAuthority] = PublicKey.findProgramAddressSync(
//       [Buffer.from("global_operational_authority")],
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

//     // Получаем адреса для токен-аккаунтов с PDA владельцами
//     tokenMarketingAccount = await getAssociatedTokenAddress(
//       mint.publicKey,
//       marketingAuthority,
//       true // allowOwnerOffCurve для PDA
//     );

//     tokenOperationalAccount = await getAssociatedTokenAddress(
//       mint.publicKey,
//       operationalAuthority,
//       true // allowOwnerOffCurve для PDA
//     );

//     bondingCurveAccount = await getAssociatedTokenAddress(
//       mint.publicKey,
//       bondingCurveAuthority,
//       true // allowOwnerOffCurve для PDA
//     );

//     // Находим PDA для бондинг кривой
//     [bondingCurvePda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("bonding_curve"), mint.publicKey.toBuffer()],
//       bondingCurveProgram.programId
//     );

//     // Находим PDA для SOL аккаунта
//     [solAccountPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("sol_account"), mint.publicKey.toBuffer()],
//       bondingCurveProgram.programId
//     );

//     // Аирдроп SOL для тестов если нужно
//     const balance = await provider.connection.getBalance(wallet.publicKey);
//     if (balance < anchor.web3.LAMPORTS_PER_SOL) {
//       await requestAirdrop(provider, wallet.publicKey, 2);
//     }
//   });

//   it("Initializes global accounts", async () => {
//     try {
//       // В обновленной программе InitializeGlobalAccounts не требует marketingAccount и operationalAccount
//       // const tx = await program.methods
//       //   .initializeGlobalAccounts()
//       //   .accounts({
//       //     admin: wallet.publicKey,
//       //     globalConfig: globalConfig,
//       //     systemProgram: anchor.web3.SystemProgram.programId,
//       //     rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//       //   })
//       //   .rpc();

//       const instruction = await program.methods
//         .initializeGlobalAccounts()
//         .accounts({
//           admin: wallet.publicKey,
//           globalConfig: globalConfig,
//           systemProgram: anchor.web3.SystemProgram.programId,
//           rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//         })
//         .instruction();

//       const tx = new Transaction().add(instruction);
//       const txid = await provider.sendAndConfirm(tx);

//       console.log("Initialize global accounts tx:", txid);

//       // Проверяем, что аккаунты созданы
//       const globalConfigAccount = await program.account["globalConfig"].fetch(
//         globalConfig
//       );
//       globalConfig;

//       // Проверяем, что PDA соответствуют ожидаемым
//       assert.equal(
//         globalConfigAccount.marketingAuthority.toString(),
//         marketingAuthority.toString(),
//         "Marketing authority doesn't match"
//       );
//       assert.equal(
//         globalConfigAccount.operationalAuthority.toString(),
//         operationalAuthority.toString(),
//         "Operational authority doesn't match"
//       );
//       assert.equal(
//         globalConfigAccount.bondingCurveAuthority.toString(),
//         bondingCurveAuthority.toString(),
//         "Bonding curve authority doesn't match"
//       );
//     } catch (error) {
//       console.error("Error initializing global accounts:", error);
//       throw error;
//     }
//   });

//   it("Creates a new user token with bonding curve", async () => {
//     try {
//       const totalSupply = new BN(1_000_000_000).mul(
//         new BN(10).pow(new BN(TOKEN_DECIMALS))
//       );

//       // Получаем глобальный конфиг для получения admin
//       const globalConfigAccount = await program.account["globalConfig"].fetch(
//         globalConfig
//       );
//       const globalAdmin = globalConfigAccount.admin;

//       console.log("Global admin pubkey:", globalAdmin.toString());
//       console.log(
//         "Marketing authority:",
//         globalConfigAccount.marketingAuthority.toString()
//       );
//       console.log(
//         "Operational authority:",
//         globalConfigAccount.operationalAuthority.toString()
//       );
//       console.log(
//         "Bonding curve authority:",
//         globalConfigAccount.bondingCurveAuthority.toString()
//       );

//       console.log("Founder account:", founderAccount.toString());
//       console.log("Token marketing account:", tokenMarketingAccount.toString());
//       console.log(
//         "Token operational account:",
//         tokenOperationalAccount.toString()
//       );
//       console.log("Bonding curve account:", bondingCurveAccount.toString());

//       const tx = await program.methods
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
//           founderAccount: founderAccount,
//           globalConfig: globalConfig,
//           tokenMarketingAccount: tokenMarketingAccount,
//           marketingAuthority: marketingAuthority,
//           tokenOperationalAccount: tokenOperationalAccount,
//           operationalAuthority: operationalAuthority,
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
//         .signers([mint])
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

//       const tokenInfoAccount = await program.account["tokenInfo"].fetch(
//         tokenInfo
//       );
//       console.log("Token info:", {
//         mint: tokenInfoAccount.mint.toString(),
//         authority: tokenInfoAccount.authority.toString(),
//         totalSupply: new BN(tokenInfoAccount.totalSupply.toString())
//           .div(new BN(10).pow(new BN(TOKEN_DECIMALS)))
//           .toString(),
//       });

//       // Проверяем, что бондинг кривая не инициализирована
//       assert(
//         !tokenInfoAccount.bondingCurveInitialized,
//         "Bonding curve should not be initialized yet"
//       );
//       assert(
//         tokenInfoAccount.bondingCurveAllocation.eq(
//           new BN(totalSupply).muln(3).divn(10)
//         ),
//         "Incorrect bonding curve allocation"
//       );

//       // Инициализируем бондинг кривую
//       const initBondingCurveTx = await program.methods
//         .initializeBondingCurve()
//         .accounts({
//           tokenInfo: tokenInfo,
//           authority: wallet.publicKey,
//           mint: mint.publicKey,
//           bondingCurve: bondingCurvePda,
//           solAccount: solAccountPda,
//           bondingCurveAuthority: bondingCurveAuthority,
//           bondingCurveTokenAccount: bondingCurveAccount,
//           bondingCurveProgram: bondingCurveProgram.programId,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//           systemProgram: anchor.web3.SystemProgram.programId,
//           rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//         })
//         .rpc();

//       console.log("Initialize bonding curve tx:", initBondingCurveTx);

//       // Проверяем, что бондинг кривая инициализирована
//       const updatedTokenInfo = await program.account["tokenInfo"].fetch(
//         tokenInfo
//       );
//       assert(
//         updatedTokenInfo.bondingCurveInitialized,
//         "Bonding curve should be initialized"
//       );

//       // Проверяем баланс SOL в бондинг кривой
//       const solBalance = await provider.connection.getBalance(solAccountPda);
//       assert.equal(
//         solBalance,
//         500_000_000,
//         "Incorrect SOL balance in bonding curve"
//       ); // 0.5 SOL

//       // Проверяем баланс токенов в бондинг кривой
//       const bondingCurveTokenBalance =
//         await provider.connection.getTokenAccountBalance(bondingCurveAccount);
//       assert(
//         new BN(bondingCurveTokenBalance.value.amount).eq(
//           tokenInfoAccount.bondingCurveAllocation
//         ),
//         "Incorrect token balance in bonding curve"
//       );
//     } catch (error) {
//       console.error("Error creating user token:", error);
//       throw error;
//     }
//   });

//   it("Can use tokens from marketing account", async () => {
//     try {
//       // Создаем адрес для назначения (куда будем переводить токены)
//       const destinationKeypair = Keypair.generate();
//       await requestAirdrop(provider, destinationKeypair.publicKey, 0.1);

//       // Создаем ассоциированный токен-аккаунт для назначения
//       const destinationTokenAccount = await getAssociatedTokenAddress(
//         mint.publicKey,
//         destinationKeypair.publicKey
//       );

//       // Создаем и отправляем инструкцию для создания ATA напрямую через соланный API
//       // Важно использовать правильный токен-программу здесь
//       try {
//         const ix = createAssociatedTokenAccountInstruction(
//           wallet.publicKey,
//           destinationTokenAccount,
//           destinationKeypair.publicKey,
//           mint.publicKey
//         );

//         const tx = new Transaction().add(ix);
//         await provider.sendAndConfirm(tx);
//         console.log(
//           `Created ATA ${destinationTokenAccount.toString()} for ${destinationKeypair.publicKey.toString()}`
//         );
//       } catch (e) {
//         console.log("ATA already exists or error:", e);
//       }

//       // Получаем текущий баланс маркетингового аккаунта
//       let marketingBalance;
//       try {
//         marketingBalance = await provider.connection.getTokenAccountBalance(
//           tokenMarketingAccount
//         );
//         console.log(
//           "Initial marketing balance:",
//           marketingBalance.value.uiAmountString
//         );
//       } catch (e) {
//         console.error("Error checking marketing balance:", e);
//         return; // Если не можем получить баланс, пропускаем тест
//       }

//       // Используем 10% токенов из маркетингового аккаунта
//       const amountToUse = new BN(marketingBalance.value.amount).divn(10);
//       console.log(
//         "Using tokens from marketing account:",
//         amountToUse.toString()
//       );

//       const tx = await program.methods
//         .useMarketingTokens(amountToUse)
//         .accounts({
//           authority: wallet.publicKey,
//           globalConfig: globalConfig,
//           tokenMarketingAccount: tokenMarketingAccount,
//           marketingAuthority: marketingAuthority,
//           mint: mint.publicKey,
//           destination: destinationTokenAccount,
//           tokenProgram: TOKEN_PROGRAM_ID,
//         })
//         .rpc();

//       console.log("Use marketing tokens tx:", tx);

//       // Проверяем новые балансы
//       const newMarketingBalance =
//         await provider.connection.getTokenAccountBalance(tokenMarketingAccount);
//       console.log(
//         "New marketing balance:",
//         newMarketingBalance.value.uiAmountString
//       );

//       const destinationBalance =
//         await provider.connection.getTokenAccountBalance(
//           destinationTokenAccount
//         );
//       console.log(
//         "Destination balance:",
//         destinationBalance.value.uiAmountString
//       );

//       // Проверяем, что токены действительно были переведены
//       assert(
//         new BN(newMarketingBalance.value.amount).lt(
//           new BN(marketingBalance.value.amount)
//         ),
//         "Marketing balance should decrease"
//       );

//       assert(
//         new BN(destinationBalance.value.amount).eq(amountToUse),
//         "Destination should receive exact amount"
//       );
//     } catch (error) {
//       console.error("Error using marketing tokens:", error);
//       throw error;
//     }
//   });

//   it("Can use tokens from operational account", async () => {
//     try {
//       // Создаем адрес для назначения (куда будем переводить токены)
//       const destinationKeypair = Keypair.generate();
//       await requestAirdrop(provider, destinationKeypair.publicKey, 0.1);

//       // Создаем ассоциированный токен-аккаунт для назначения
//       const destinationTokenAccount = await getAssociatedTokenAddress(
//         mint.publicKey,
//         destinationKeypair.publicKey
//       );

//       // Создаем и отправляем инструкцию для создания ATA
//       try {
//         const ix = createAssociatedTokenAccountInstruction(
//           wallet.publicKey,
//           destinationTokenAccount,
//           destinationKeypair.publicKey,
//           mint.publicKey
//         );

//         const tx = new Transaction().add(ix);
//         await provider.sendAndConfirm(tx);
//         console.log(
//           `Created ATA ${destinationTokenAccount.toString()} for ${destinationKeypair.publicKey.toString()}`
//         );
//       } catch (e) {
//         console.log("ATA already exists or error:", e);
//       }

//       // Получаем текущий баланс операционного аккаунта
//       let operationalBalance;
//       try {
//         operationalBalance = await provider.connection.getTokenAccountBalance(
//           tokenOperationalAccount
//         );
//         console.log(
//           "Initial operational balance:",
//           operationalBalance.value.uiAmountString
//         );
//       } catch (e) {
//         console.error("Error checking operational balance:", e);
//         return; // Если не можем получить баланс, пропускаем тест
//       }

//       // Используем 5% токенов из операционного аккаунта
//       const amountToUse = new BN(operationalBalance.value.amount).divn(20);
//       console.log(
//         "Using tokens from operational account:",
//         amountToUse.toString()
//       );

//       const tx = await program.methods
//         .useOperationalTokens(amountToUse)
//         .accounts({
//           authority: wallet.publicKey,
//           globalConfig: globalConfig,
//           tokenOperationalAccount: tokenOperationalAccount,
//           operationalAuthority: operationalAuthority,
//           mint: mint.publicKey,
//           destination: destinationTokenAccount,
//           tokenProgram: TOKEN_PROGRAM_ID,
//         })
//         .rpc();

//       console.log("Use operational tokens tx:", tx);

//       // Проверяем новые балансы
//       const newOperationalBalance =
//         await provider.connection.getTokenAccountBalance(
//           tokenOperationalAccount
//         );
//       console.log(
//         "New operational balance:",
//         newOperationalBalance.value.uiAmountString
//       );

//       const destinationBalance =
//         await provider.connection.getTokenAccountBalance(
//           destinationTokenAccount
//         );
//       console.log(
//         "Destination balance:",
//         destinationBalance.value.uiAmountString
//       );

//       // Проверяем, что токены действительно были переведены
//       assert(
//         new BN(newOperationalBalance.value.amount).lt(
//           new BN(operationalBalance.value.amount)
//         ),
//         "Operational balance should decrease"
//       );

//       assert(
//         new BN(destinationBalance.value.amount).eq(amountToUse),
//         "Destination should receive exact amount"
//       );
//     } catch (error) {
//       console.error("Error using operational tokens:", error);
//       throw error;
//     }
//   });
// });
