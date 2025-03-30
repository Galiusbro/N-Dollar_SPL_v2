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

// describe("Token Creator and Bonding Curve Test", () => {
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//   const tokenCreatorProgram = anchor.workspace.TokenCreator as Program;
//   const bondingCurveProgram = anchor.workspace.BondingCurve as Program;
//   const wallet = provider.wallet as anchor.Wallet;

//   // Ключи и адреса
//   let mint: Keypair;
//   let metadataPda: PublicKey;
//   let tokenInfo: PublicKey;
//   let founderAccount: PublicKey;
//   let bondingCurveTokenAccount: PublicKey; // Переименовано для ясности - это адрес токен аккаунта
//   let globalConfig: PublicKey;

//   // PDA для глобальных authority
//   let marketingAuthority: PublicKey;
//   let operationalAuthority: PublicKey;
//   let bondingCurveAuthority: PublicKey;

//   // Аккаунты для токенов
//   let tokenMarketingAccount: PublicKey;
//   let tokenOperationalAccount: PublicKey;

//   // Бондинг кривая
//   let bondingCurvePda: PublicKey;
//   let solAccountPda: PublicKey;

//   before(async () => {
//     console.log("Wallet public key:", wallet.publicKey.toString());

//     mint = Keypair.generate();

//     // Находим PDA для метаданных
//     [metadataPda] = PublicKey.findProgramAddressSync(
//       [
//         Buffer.from("metadata"),
//         new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBuffer(),
//         mint.publicKey.toBuffer(),
//       ],
//       new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
//     );

//     // Находим PDA для token_info
//     [tokenInfo] = PublicKey.findProgramAddressSync(
//       [Buffer.from("token_info"), mint.publicKey.toBuffer()],
//       tokenCreatorProgram.programId
//     );

//     // Находим PDA для global_config
//     [globalConfig] = PublicKey.findProgramAddressSync(
//       [Buffer.from("global_config")],
//       tokenCreatorProgram.programId
//     );

//     // Находим PDA для authority
//     [marketingAuthority] = PublicKey.findProgramAddressSync(
//       [Buffer.from("global_marketing_authority")],
//       tokenCreatorProgram.programId
//     );

//     [operationalAuthority] = PublicKey.findProgramAddressSync(
//       [Buffer.from("global_operational_authority")],
//       tokenCreatorProgram.programId
//     );

//     // Находим PDA для bonding_curve_authority с bump
//     [bondingCurveAuthority] = PublicKey.findProgramAddressSync(
//       [Buffer.from("global_bonding_curve_authority")],
//       tokenCreatorProgram.programId
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

//     // Получаем адреса ассоциированных токен аккаунтов
//     founderAccount = await getAssociatedTokenAddress(
//       mint.publicKey,
//       wallet.publicKey
//     );

//     tokenMarketingAccount = await getAssociatedTokenAddress(
//       mint.publicKey,
//       marketingAuthority,
//       true
//     );

//     tokenOperationalAccount = await getAssociatedTokenAddress(
//       mint.publicKey,
//       operationalAuthority,
//       true
//     );

//     bondingCurveTokenAccount = await getAssociatedTokenAddress(
//       mint.publicKey,
//       bondingCurveAuthority,
//       true
//     );

//     // Аирдроп SOL если нужно
//     const balance = await provider.connection.getBalance(wallet.publicKey);
//     if (balance < 2 * anchor.web3.LAMPORTS_PER_SOL) {
//       await requestAirdrop(provider, wallet.publicKey, 2);
//     }
//   });

//   it("Initializes global accounts", async () => {
//     try {
//       const tx = await tokenCreatorProgram.methods
//         .initializeGlobalAccounts()
//         .accounts({
//           admin: wallet.publicKey,
//           globalConfig: globalConfig,
//           systemProgram: anchor.web3.SystemProgram.programId,
//           rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//         })
//         .rpc();

//       console.log("Initialize global accounts tx:", tx);

//       //   const globalConfigAccount =
//       //     await tokenCreatorProgram.account.globalConfig.fetch(globalConfig);

//       const globalConfigAccount = await tokenCreatorProgram.account[
//         "globalConfig"
//       ].fetch(globalConfig);

//       assert.equal(
//         globalConfigAccount.marketingAuthority.toString(),
//         marketingAuthority.toString()
//       );
//       assert.equal(
//         globalConfigAccount.operationalAuthority.toString(),
//         operationalAuthority.toString()
//       );
//       assert.equal(
//         globalConfigAccount.bondingCurveAuthority.toString(),
//         bondingCurveAuthority.toString()
//       );
//     } catch (error) {
//       console.error("Error initializing global accounts:", error);
//       throw error;
//     }
//   });

//   it("Creates a new user token", async () => {
//     try {
//       const totalSupply = new BN(1_000_000_000).mul(
//         new BN(10).pow(new BN(TOKEN_DECIMALS))
//       );

//       //   const globalConfigAccount =
//       //     await tokenCreatorProgram.account.globalConfig.fetch(globalConfig);

//       const globalConfigAccount = await tokenCreatorProgram.account[
//         "globalConfig"
//       ].fetch(globalConfig);

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
//           founderAccount: founderAccount,
//           globalConfig: globalConfig,
//           tokenMarketingAccount: tokenMarketingAccount,
//           marketingAuthority: marketingAuthority,
//           tokenOperationalAccount: tokenOperationalAccount,
//           operationalAuthority: operationalAuthority,
//           globalAdmin: globalConfigAccount.admin,
//           bondingCurveAccount: bondingCurveTokenAccount,
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

//       // Проверяем балансы
//       const balances = await Promise.all([
//         provider.connection.getTokenAccountBalance(founderAccount),
//         provider.connection.getTokenAccountBalance(tokenMarketingAccount),
//         provider.connection.getTokenAccountBalance(tokenOperationalAccount),
//         provider.connection.getTokenAccountBalance(bondingCurveTokenAccount),
//       ]);

//       console.log("Founder balance:", balances[0].value.uiAmountString);
//       console.log("Marketing balance:", balances[1].value.uiAmountString);
//       console.log("Operational balance:", balances[2].value.uiAmountString);
//       console.log("Bonding curve balance:", balances[3].value.uiAmountString);

//       //   const tokenInfoAccount =
//       //     await tokenCreatorProgram.account.tokenInfo.fetch(tokenInfo);

//       const tokenInfoAccount = await tokenCreatorProgram.account[
//         "tokenInfo"
//       ].fetch(tokenInfo);

//       console.log("Token info:", {
//         mint: tokenInfoAccount.mint.toString(),
//         authority: tokenInfoAccount.authority.toString(),
//         totalSupply: tokenInfoAccount.totalSupply.toString(),
//         bondingCurveAllocation:
//           tokenInfoAccount.bondingCurveAllocation.toString(),
//       });
//     } catch (error) {
//       console.error("Error creating user token:", error);
//       throw error;
//     }
//   });

//   it("Initializes bonding curve", async () => {
//     try {
//       const tx = await tokenCreatorProgram.methods
//         .initializeBondingCurve()
//         .accounts({
//           tokenInfo: tokenInfo,
//           authority: wallet.publicKey,
//           mint: mint.publicKey,
//           bondingCurve: bondingCurvePda,
//           solAccount: solAccountPda,
//           bondingCurveAuthority: bondingCurveAuthority,
//           bondingCurveTokenAccount: bondingCurveTokenAccount,
//           // Добавляем ссылку на программу token_creator
//           tokenCreatorProgram: tokenCreatorProgram.programId,
//           bondingCurveProgram: bondingCurveProgram.programId,
//           authorityProgram: tokenCreatorProgram.programId,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//           systemProgram: anchor.web3.SystemProgram.programId,
//           rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//         })
//         .rpc();

//       console.log("Initialize bonding curve tx:", tx);

//       // Проверяем, что бондинг кривая инициализирована
//       const tokenInfoAccount = await tokenCreatorProgram.account[
//         "tokenInfo"
//       ].fetch(tokenInfo);
//       assert.isTrue(tokenInfoAccount.bondingCurveInitialized);

//       // Проверяем баланс SOL в бондинг кривой
//       const solBalance = await provider.connection.getBalance(solAccountPda);
//       assert.isAtLeast(solBalance, 500_000_000); // 0.5 SOL

//       // Проверяем баланс токенов в бондинг кривой
//       const tokenBalance = await provider.connection.getTokenAccountBalance(
//         bondingCurveTokenAccount
//       );
//       assert.equal(
//         tokenBalance.value.amount,
//         tokenInfoAccount.bondingCurveAllocation.toString()
//       );
//     } catch (error) {
//       console.error("Error initializing bonding curve:", error);
//       throw error;
//     }
//   });

//   // Функция для покупки токенов
//   // Функция для покупки токенов
//   it("Can buy tokens from bonding curve", async () => {
//     try {
//       // Создаем аккаунт для покупателя
//       const buyer = Keypair.generate();
//       await requestAirdrop(provider, buyer.publicKey, 1); // 1 SOL

//       // Создаем токен-аккаунт для покупателя
//       const buyerTokenAccount = await getAssociatedTokenAddress(
//         mint.publicKey,
//         buyer.publicKey
//       );

//       const createAtaIx = createAssociatedTokenAccountInstruction(
//         wallet.publicKey,
//         buyerTokenAccount,
//         buyer.publicKey,
//         mint.publicKey
//       );

//       // Выводим адреса для отладки
//       console.log("bondingCurveAuthority:", bondingCurveAuthority.toString());
//       console.log(
//         "tokenCreatorProgram ID:",
//         tokenCreatorProgram.programId.toString()
//       );
//       console.log(
//         "bondingCurveProgram ID:",
//         bondingCurveProgram.programId.toString()
//       );

//       // Покупаем токены
//       const amountSol = new BN(0.1 * anchor.web3.LAMPORTS_PER_SOL); // 0.1 SOL

//       // Устанавливаем большой лимит вычислений для отладки
//       const computeBudgetIx =
//         anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
//           units: 400_000,
//         });

//       // Проверяем баланс токенов на аккаунте бондинг кривой перед покупкой
//       try {
//         const bondingCurveTokenBalance =
//           await provider.connection.getTokenAccountBalance(
//             bondingCurveTokenAccount
//           );
//         console.log(
//           "Bonding curve token balance before purchase:",
//           bondingCurveTokenBalance.value.uiAmountString
//         );
//       } catch (e) {
//         console.log("Error getting bonding curve token balance:", e);
//       }

//       // Получаем данные из аккаунта бондинг кривой перед покупкой
//       try {
//         const bondingCurveData =
//           await bondingCurveProgram.account.bondingCurve.fetch(bondingCurvePda);
//         console.log("Bonding curve before buy:", {
//           mint: bondingCurveData.mint.toString(),
//           authority: bondingCurveData.authority.toString(),
//           tokensSold: bondingCurveData.tokensSold.toString(),
//           solCollected: bondingCurveData.solCollected.toString(),
//         });
//       } catch (e) {
//         console.log("Error fetching bonding curve account:", e);
//       }

//       // Используем существующую функцию с 2 аргументами
//       const tx = await bondingCurveProgram.methods
//         .buyTokens(amountSol)
//         .accounts({
//           bondingCurve: bondingCurvePda,
//           mint: mint.publicKey,
//           buyer: buyer.publicKey,
//           buyerTokenAccount: buyerTokenAccount,
//           bondingCurveTokenAccount: bondingCurveTokenAccount,
//           bondingCurveAuthority: bondingCurveAuthority,
//           solAccount: solAccountPda,
//           tokenCreatorProgram: tokenCreatorProgram.programId,
//           authorityProgram: tokenCreatorProgram.programId,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           systemProgram: anchor.web3.SystemProgram.programId,
//         })
//         .preInstructions([computeBudgetIx, createAtaIx])
//         .signers([buyer])
//         .rpc({ commitment: "confirmed" });

//       console.log("Buy tokens tx:", tx);

//       // Получаем данные из аккаунта бондинг кривой после покупки
//       const bondingCurveData =
//         await bondingCurveProgram.account.bondingCurve.fetch(bondingCurvePda);
//       console.log("Bonding curve after buy:", {
//         tokensSold: bondingCurveData.tokensSold.toString(),
//         solCollected: bondingCurveData.solCollected.toString(),
//       });

//       // Проверяем баланс покупателя
//       const buyerBalance = await provider.connection.getTokenAccountBalance(
//         buyerTokenAccount
//       );
//       console.log("Buyer token balance:", buyerBalance.value.uiAmountString);
//       assert(
//         Number(buyerBalance.value.amount) > 0,
//         "Buyer should receive tokens"
//       );
//     } catch (error) {
//       console.error("Error buying tokens:", error);

//       // Выводим детали ошибки для более глубокого анализа
//       if (error.logs) {
//         console.error("Error logs:", error.logs);
//       }

//       throw error;
//     }
//   });

//   // Функция для продажи токенов
//   it("Can sell tokens to bonding curve", async () => {
//     try {
//       // Создаем нового продавца
//       const seller = Keypair.generate();
//       await requestAirdrop(provider, seller.publicKey, 0.5); // 0.5 SOL для газа

//       const sellerTokenAccount = await getAssociatedTokenAddress(
//         mint.publicKey,
//         seller.publicKey
//       );

//       // Создаем ATA для продавца
//       const createAtaIx = createAssociatedTokenAccountInstruction(
//         wallet.publicKey,
//         sellerTokenAccount,
//         seller.publicKey,
//         mint.publicKey
//       );

//       // Сначала нужно купить токены, которые потом будем продавать
//       // Устанавливаем большой лимит вычислений
//       const computeBudgetIx =
//         anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
//           units: 400_000,
//         });

//       // Сначала покупаем токены
//       const buyAmount = new BN(0.2 * anchor.web3.LAMPORTS_PER_SOL); // 0.2 SOL для покупки

//       // Покупка токенов
//       const buyTx = await bondingCurveProgram.methods
//         .buyTokens(buyAmount)
//         .accounts({
//           bondingCurve: bondingCurvePda,
//           mint: mint.publicKey,
//           buyer: seller.publicKey, // Используем продавца как покупателя для первой транзакции
//           buyerTokenAccount: sellerTokenAccount,
//           bondingCurveTokenAccount: bondingCurveTokenAccount,
//           bondingCurveAuthority: bondingCurveAuthority,
//           solAccount: solAccountPda,
//           tokenCreatorProgram: tokenCreatorProgram.programId,
//           authorityProgram: tokenCreatorProgram.programId,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           systemProgram: anchor.web3.SystemProgram.programId,
//         })
//         .preInstructions([computeBudgetIx, createAtaIx])
//         .signers([seller])
//         .rpc({ commitment: "confirmed" });

//       console.log("Initial buy tokens tx:", buyTx);

//       // Получаем баланс токенов после покупки
//       const tokenBalance = await provider.connection.getTokenAccountBalance(
//         sellerTokenAccount
//       );
//       console.log(
//         "Token balance before selling:",
//         tokenBalance.value.uiAmountString
//       );
//       assert(Number(tokenBalance.value.amount) > 0, "No tokens to sell");

//       const sellAmount = new BN(
//         Math.floor(Number(tokenBalance.value.amount) / 2)
//       ); // Продаем половину
//       console.log("Selling amount:", sellAmount.toString());

//       // Продаем токены
//       const sellTx = await bondingCurveProgram.methods
//         .sellTokens(sellAmount)
//         .accounts({
//           bondingCurve: bondingCurvePda,
//           mint: mint.publicKey,
//           seller: seller.publicKey,
//           sellerTokenAccount: sellerTokenAccount,
//           bondingCurveTokenAccount: bondingCurveTokenAccount,
//           bondingCurveAuthority: bondingCurveAuthority,
//           solAccount: solAccountPda,
//           tokenCreatorProgram: tokenCreatorProgram.programId,
//           authorityProgram: tokenCreatorProgram.programId,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           systemProgram: anchor.web3.SystemProgram.programId,
//         })
//         .signers([seller])
//         .rpc({ commitment: "confirmed" });

//       console.log("Sell tokens tx:", sellTx);

//       const bondingCurveDataAfterSell =
//         await bondingCurveProgram.account.bondingCurve.fetch(bondingCurvePda);
//       console.log("Bonding curve after sell:", {
//         tokensSold: bondingCurveDataAfterSell.tokensSold.toString(),
//         solCollected: bondingCurveDataAfterSell.solCollected.toString(),
//       });

//       // Проверяем новый баланс
//       const newBalance = await provider.connection.getTokenAccountBalance(
//         sellerTokenAccount
//       );
//       console.log(
//         "Seller token balance after sell:",
//         newBalance.value.uiAmountString
//       );
//       assert(
//         Number(newBalance.value.amount) < Number(tokenBalance.value.amount),
//         "Token amount should decrease after selling"
//       );
//     } catch (error) {
//       console.error("Error selling tokens:", error);

//       // Выводим детали ошибки для более глубокого анализа
//       if (error.logs) {
//         console.error("Error logs:", error.logs);
//       }

//       throw error;
//     }
//   });
// });
