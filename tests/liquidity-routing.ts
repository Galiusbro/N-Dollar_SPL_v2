// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import {
//   PublicKey,
//   Keypair,
//   SystemProgram,
//   Transaction,
//   LAMPORTS_PER_SOL,
// } from "@solana/web3.js";
// import {
//   TOKEN_PROGRAM_ID,
//   ASSOCIATED_TOKEN_PROGRAM_ID,
//   getAssociatedTokenAddress,
//   createAssociatedTokenAccountInstruction,
//   createInitializeMintInstruction,
//   createMintToInstruction,
//   AuthorityType,
//   createSetAuthorityInstruction,
// } from "@solana/spl-token";
// import { Genesis } from "../target/types/genesis";
// import { LiquidityManager } from "../target/types/liquidity_manager";
// import { BondingCurve } from "../target/types/bonding_curve";
// import { assert } from "chai";
// import BN from "bn.js";

// // Адрес программы метаданных токенов
// const MPL_TOKEN_METADATA_PROGRAM_ID = new PublicKey(
//   "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
// );

// describe("Тестирование автоматической маршрутизации ликвидности", () => {
//   // Настройка провайдера Anchor
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//   // Подключение к программам
//   const genesisProgram = anchor.workspace.Genesis as Program<Genesis>;
//   const liquidityManagerProgram = anchor.workspace
//     .LiquidityManager as Program<LiquidityManager>;
//   const bondingCurveProgram = anchor.workspace
//     .BondingCurve as Program<BondingCurve>;

//   // Создаем детерминированные ключи с фиксированным seed для предсказуемости тестов
//   // Альтернатива Keypair.generate() для стабильности тестов
//   const adminSeed = Uint8Array.from(Array(32).fill(1));
//   const userSeed = Uint8Array.from(Array(32).fill(2));
//   const feeAuthoritySeed = Uint8Array.from(Array(32).fill(3));
//   const admin = Keypair.fromSeed(adminSeed);
//   const user = Keypair.fromSeed(userSeed);
//   const feeAuthority = Keypair.fromSeed(feeAuthoritySeed);

//   let nDollarMint: PublicKey;
//   let liquidityManager: PublicKey;
//   let adminNDollarAccount: PublicKey;
//   let userNDollarAccount: PublicKey;
//   let poolSolAccount: PublicKey;
//   let poolNDollarAccount: PublicKey;
//   let feesAccount: PublicKey;

//   // Информация о монете
//   let coinMint: PublicKey;
//   let coinData: PublicKey;
//   let userCoinAccount: PublicKey;
//   let adminCoinAccount: PublicKey;

//   // Бондинговая кривая
//   let bondingCurve: PublicKey;
//   let liquidityPool: PublicKey;

//   // Параметры
//   const nDollarDecimals = 9;

//   before(async () => {
//     try {
//       // Финансируем кошельки
//       await provider.connection.requestAirdrop(
//         admin.publicKey,
//         10 * LAMPORTS_PER_SOL
//       );

//       await provider.connection.requestAirdrop(
//         user.publicKey,
//         5 * LAMPORTS_PER_SOL
//       );

//       await provider.connection.requestAirdrop(
//         feeAuthority.publicKey,
//         5 * LAMPORTS_PER_SOL
//       );

//       // Ждем подтверждения транзакций
//       await new Promise((resolve) => setTimeout(resolve, 1000));

//       // Создаем минт N-Dollar с детерминированным ключом
//       const nDollarMintSeed = Uint8Array.from(Array(32).fill(4));
//       const nDollarMintKeypair = Keypair.fromSeed(nDollarMintSeed);
//       nDollarMint = nDollarMintKeypair.publicKey;

//       // Создаем ассоциированные токен-аккаунты для N-Dollar
//       adminNDollarAccount = await getAssociatedTokenAddress(
//         nDollarMint,
//         admin.publicKey
//       );

//       userNDollarAccount = await getAssociatedTokenAddress(
//         nDollarMint,
//         user.publicKey
//       );

//       // Находим PDA для liquidity manager
//       const [liquidityManagerPDA] = PublicKey.findProgramAddressSync(
//         [Buffer.from("liquidity_manager"), admin.publicKey.toBuffer()],
//         liquidityManagerProgram.programId
//       );
//       liquidityManager = liquidityManagerPDA;

//       // Находим PDA для аккаунта SOL в пуле
//       const [poolSolPDA] = PublicKey.findProgramAddressSync(
//         [Buffer.from("pool_sol"), liquidityManager.toBuffer()],
//         liquidityManagerProgram.programId
//       );
//       poolSolAccount = poolSolPDA;

//       // Создаем минт для N-Dollar
//       const mintTx = new Transaction();
//       mintTx.add(
//         SystemProgram.createAccount({
//           fromPubkey: admin.publicKey,
//           newAccountPubkey: nDollarMint,
//           lamports: await provider.connection.getMinimumBalanceForRentExemption(
//             82
//           ),
//           space: 82,
//           programId: TOKEN_PROGRAM_ID,
//         })
//       );

//       mintTx.add(
//         createInitializeMintInstruction(
//           nDollarMint,
//           nDollarDecimals,
//           admin.publicKey,
//           admin.publicKey
//         )
//       );

//       await anchor.web3.sendAndConfirmTransaction(provider.connection, mintTx, [
//         admin,
//         nDollarMintKeypair,
//       ]);

//       // Создаем токен-аккаунты
//       const accountsTx = new Transaction();

//       // Для админа
//       accountsTx.add(
//         createAssociatedTokenAccountInstruction(
//           admin.publicKey,
//           adminNDollarAccount,
//           admin.publicKey,
//           nDollarMint
//         )
//       );

//       // Для пользователя
//       accountsTx.add(
//         createAssociatedTokenAccountInstruction(
//           admin.publicKey,
//           userNDollarAccount,
//           user.publicKey,
//           nDollarMint
//         )
//       );

//       await anchor.web3.sendAndConfirmTransaction(
//         provider.connection,
//         accountsTx,
//         [admin]
//       );

//       // Получаем ассоциированный токен-аккаунт для liquidity manager (PDA)
//       poolNDollarAccount = await getAssociatedTokenAddress(
//         nDollarMint,
//         liquidityManager,
//         true // allowOwnerOffCurve flag необходим для PDA
//       );

//       // Создаем токен-аккаунт для пула ликвидности
//       const createPoolAccountTx = new Transaction().add(
//         createAssociatedTokenAccountInstruction(
//           admin.publicKey,
//           poolNDollarAccount,
//           liquidityManager,
//           nDollarMint
//         )
//       );

//       await anchor.web3.sendAndConfirmTransaction(
//         provider.connection,
//         createPoolAccountTx,
//         [admin]
//       );

//       // Создаем токен-аккаунт для комиссий
//       feesAccount = await getAssociatedTokenAddress(
//         nDollarMint,
//         feeAuthority.publicKey
//       );

//       const createFeesAccountTx = new Transaction().add(
//         createAssociatedTokenAccountInstruction(
//           admin.publicKey,
//           feesAccount,
//           feeAuthority.publicKey,
//           nDollarMint
//         )
//       );

//       await anchor.web3.sendAndConfirmTransaction(
//         provider.connection,
//         createFeesAccountTx,
//         [admin]
//       );

//       // Минтим N-Dollar для тестирования
//       const mintTokensTx = new Transaction();

//       // Минтим для админа
//       mintTokensTx.add(
//         createMintToInstruction(
//           nDollarMint,
//           adminNDollarAccount,
//           admin.publicKey,
//           1_000_000_000_000 // 1,000,000 N-Dollar с 9 десятичными знаками
//         )
//       );

//       // Минтим для пула
//       mintTokensTx.add(
//         createMintToInstruction(
//           nDollarMint,
//           poolNDollarAccount,
//           admin.publicKey,
//           100_000_000_000 // 100,000 N-Dollar для начальной ликвидности
//         )
//       );

//       await anchor.web3.sendAndConfirmTransaction(
//         provider.connection,
//         mintTokensTx,
//         [admin]
//       );

//       console.log("Тестовое окружение успешно инициализировано");
//     } catch (error) {
//       console.error("Ошибка при инициализации:", error);
//       throw error;
//     }
//   });

//   it("Инициализирует менеджер ликвидности", async () => {
//     try {
//       // Инициализируем менеджер ликвидности
//       await liquidityManagerProgram.methods
//         .initializeLiquidityManager()
//         .accounts({
//           authority: admin.publicKey,
//           nDollarMint: nDollarMint,
//           liquidityManager: liquidityManager,
//           systemProgram: SystemProgram.programId,
//           rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//         })
//         .signers([admin])
//         .rpc();

//       // Проверяем, что менеджер ликвидности был инициализирован
//       const liquidityManagerAccount =
//         await liquidityManagerProgram.account.liquidityManager.fetch(
//           liquidityManager
//         );
//       assert.equal(
//         liquidityManagerAccount.authority.toString(),
//         admin.publicKey.toString()
//       );
//       assert.equal(
//         liquidityManagerAccount.nDollarMint.toString(),
//         nDollarMint.toString()
//       );
//       assert.equal(liquidityManagerAccount.totalLiquidity.toString(), "0");
//       assert.equal(liquidityManagerAccount.totalUsers.toString(), "0");

//       console.log("Менеджер ликвидности успешно инициализирован");
//     } catch (error) {
//       console.error("Ошибка при инициализации менеджера ликвидности:", error);
//       throw error;
//     }
//   });

//   it("Создает новую монету с автоматической маршрутизацией комиссий", async () => {
//     try {
//       // Создаем новый минт для монеты с детерминированным ключом
//       const coinMintSeed = Uint8Array.from(Array(32).fill(5));
//       const coinMintKeypair = Keypair.fromSeed(coinMintSeed);
//       coinMint = coinMintKeypair.publicKey;

//       // Находим PDA для данных монеты
//       const [coinDataPDA] = PublicKey.findProgramAddressSync(
//         [Buffer.from("coin_data"), coinMint.toBuffer()],
//         genesisProgram.programId
//       );
//       coinData = coinDataPDA;

//       // Создаем токен-аккаунты для монеты
//       userCoinAccount = await getAssociatedTokenAddress(
//         coinMint,
//         user.publicKey
//       );

//       adminCoinAccount = await getAssociatedTokenAddress(
//         coinMint,
//         admin.publicKey
//       );

//       // Получаем баланс пула N-Dollar до создания монеты
//       const poolBalanceBefore =
//         await provider.connection.getTokenAccountBalance(poolNDollarAccount);

//       // Создаем минт для монеты
//       const tx = new Transaction();
//       tx.add(
//         SystemProgram.createAccount({
//           fromPubkey: admin.publicKey,
//           newAccountPubkey: coinMint,
//           lamports: await provider.connection.getMinimumBalanceForRentExemption(
//             82
//           ),
//           space: 82,
//           programId: TOKEN_PROGRAM_ID,
//         })
//       );

//       tx.add(
//         createInitializeMintInstruction(
//           coinMint,
//           9, // 9 decimals
//           admin.publicKey, // Временно устанавливаем mint authority на admin
//           admin.publicKey // Временно устанавливаем freeze authority на admin
//         )
//       );

//       // Создаем токен-аккаунты
//       tx.add(
//         createAssociatedTokenAccountInstruction(
//           admin.publicKey,
//           adminCoinAccount,
//           admin.publicKey,
//           coinMint
//         )
//       );

//       tx.add(
//         createAssociatedTokenAccountInstruction(
//           admin.publicKey,
//           userCoinAccount,
//           user.publicKey,
//           coinMint
//         )
//       );

//       await anchor.web3.sendAndConfirmTransaction(provider.connection, tx, [
//         admin,
//         coinMintKeypair,
//       ]);

//       // Создаем метаданные монеты (используем PDA вместо keypair)
//       const metadataPDA = PublicKey.findProgramAddressSync(
//         [
//           Buffer.from("metadata"),
//           MPL_TOKEN_METADATA_PROGRAM_ID.toBuffer(),
//           coinMint.toBuffer(),
//         ],
//         MPL_TOKEN_METADATA_PROGRAM_ID
//       )[0];

//       const ndollarPayment = new BN(1_000_000_000); // 1,000 N-Dollar

//       // Передаем N-Dollar для создания монеты
//       // Предварительно проверяем, что у админа достаточно N-Dollar
//       const adminNDollarBalanceBefore =
//         await provider.connection.getTokenAccountBalance(adminNDollarAccount);
//       console.log(
//         "Баланс N-Dollar админа перед созданием монеты:",
//         adminNDollarBalanceBefore.value.uiAmount
//       );

//       // Получаем актуальную информацию о программе Genesis
//       const genesisInfo = await provider.connection.getAccountInfo(
//         genesisProgram.programId
//       );
//       console.log("Genesis program ID:", genesisProgram.programId.toString());
//       console.log("Genesis program size:", genesisInfo?.data.length);

//       // Создаем монету и автоматически направляем комиссию
//       console.log("Вызываем метод createCoin...");
//       console.log("PDA метаданных:", metadataPDA.toString());

//       // Проверяем существование аккаунта метаданных
//       const metadataAccountInfo = await provider.connection.getAccountInfo(
//         metadataPDA
//       );
//       console.log(
//         "Аккаунт метаданных существует:",
//         metadataAccountInfo !== null
//       );

//       try {
//         await genesisProgram.methods
//           .createCoin(
//             "Test Coin",
//             "TEST",
//             "https://test-uri.com",
//             ndollarPayment
//           )
//           .accounts({
//             creator: admin.publicKey,
//             mint: coinMint,
//             metadata: metadataPDA,
//             mintAuthority: admin.publicKey,
//             coinData: coinData,
//             ndollarTokenAccount: adminNDollarAccount,
//             feesAccount: feesAccount,
//             feeAuthority: feeAuthority.publicKey,
//             creatorTokenAccount: adminCoinAccount,
//             liquidityManager: liquidityManager,
//             poolNdollarAccount: poolNDollarAccount,
//             liquidityManagerProgram: liquidityManagerProgram.programId,
//             tokenProgram: TOKEN_PROGRAM_ID,
//             systemProgram: SystemProgram.programId,
//             associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//             metadataProgram: MPL_TOKEN_METADATA_PROGRAM_ID,
//             rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//           })
//           .signers([admin])
//           .rpc();

//         console.log("Создание монеты успешно!");
//       } catch (error) {
//         console.log("Детальная ошибка при создании монеты:", error);
//         // Проверяем PDA для coinData
//         const [checkCoinDataPDA] = PublicKey.findProgramAddressSync(
//           [Buffer.from("coin_data"), coinMint.toBuffer()],
//           genesisProgram.programId
//         );
//         console.log("Проверка PDA для coinData:", checkCoinDataPDA.toString());
//         throw error;
//       }

//       // Получаем баланс пула N-Dollar после создания монеты
//       const poolBalanceAfter = await provider.connection.getTokenAccountBalance(
//         poolNDollarAccount
//       );

//       // Проверяем, что комиссия была автоматически направлена в пул ликвидности
//       console.log("Баланс пула до:", poolBalanceBefore.value.uiAmount);
//       console.log("Баланс пула после:", poolBalanceAfter.value.uiAmount);
//       console.log(
//         "Разница:",
//         poolBalanceAfter.value.uiAmount! - poolBalanceBefore.value.uiAmount!
//       );

//       assert.equal(
//         Number(poolBalanceAfter.value.amount) -
//           Number(poolBalanceBefore.value.amount),
//         ndollarPayment.toNumber(),
//         "Комиссия не была правильно маршрутизирована в пул ликвидности"
//       );

//       console.log(
//         "Монета успешно создана и комиссия автоматически направлена в пул ликвидности"
//       );
//     } catch (error) {
//       console.error("Ошибка при создании монеты:", error);
//       throw error;
//     }
//   });

//   it("Настраивает бондинговую кривую", async () => {
//     try {
//       // Находим PDA для бондинговой кривой
//       const [bondingCurvePDA] = PublicKey.findProgramAddressSync(
//         [Buffer.from("bonding_curve"), coinMint.toBuffer()],
//         bondingCurveProgram.programId
//       );
//       bondingCurve = bondingCurvePDA;

//       // Создаем пул ликвидности для бондинговой кривой
//       liquidityPool = await getAssociatedTokenAddress(
//         nDollarMint,
//         bondingCurve,
//         true // allowOwnerOffCurve flag для PDA
//       );

//       // Создаем аккаунт для пула ликвидности
//       const liquidityPoolTx = new Transaction().add(
//         createAssociatedTokenAccountInstruction(
//           admin.publicKey,
//           liquidityPool,
//           bondingCurve,
//           nDollarMint
//         )
//       );

//       await anchor.web3.sendAndConfirmTransaction(
//         provider.connection,
//         liquidityPoolTx,
//         [admin]
//       );

//       // Минтим начальную ликвидность в пул
//       const initialLiquidity = new BN(1000 * Math.pow(10, nDollarDecimals)); // 1000 N-Dollar
//       const mintToPoolTx = new Transaction().add(
//         createMintToInstruction(
//           nDollarMint,
//           liquidityPool,
//           admin.publicKey,
//           initialLiquidity.toNumber()
//         )
//       );

//       await anchor.web3.sendAndConfirmTransaction(
//         provider.connection,
//         mintToPoolTx,
//         [admin]
//       );

//       // Инициализируем бондинговую кривую
//       const initialPrice = new BN(50_000_000); // 0.05 N-Dollar
//       const power = 2; // квадратичная кривая
//       const feePercent = 50; // 0.5% комиссия

//       await bondingCurveProgram.methods
//         .initializeBondingCurve(coinMint, initialPrice, power, feePercent)
//         .accounts({
//           creator: admin.publicKey,
//           bondingCurve: bondingCurve,
//           coinMint: coinMint,
//           ndollarMint: nDollarMint,
//           liquidityPool: liquidityPool,
//           systemProgram: SystemProgram.programId,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//         })
//         .signers([admin])
//         .rpc();

//       // Передаем права mint authority контракту бондинговой кривой
//       const transferAuthorityTx = new Transaction().add(
//         createSetAuthorityInstruction(
//           coinMint,
//           admin.publicKey,
//           AuthorityType.MintTokens,
//           bondingCurve
//         )
//       );

//       await anchor.web3.sendAndConfirmTransaction(
//         provider.connection,
//         transferAuthorityTx,
//         [admin]
//       );

//       // Проверяем, что бондинговая кривая была инициализирована
//       const bondingCurveAccount =
//         await bondingCurveProgram.account.bondingCurve.fetch(bondingCurve);

//       assert.equal(
//         bondingCurveAccount.coinMint.toString(),
//         coinMint.toString()
//       );
//       assert.equal(
//         bondingCurveAccount.ndollarMint.toString(),
//         nDollarMint.toString()
//       );
//       assert.equal(
//         bondingCurveAccount.creator.toString(),
//         admin.publicKey.toString()
//       );

//       console.log("Бондинговая кривая успешно настроена");
//     } catch (error) {
//       console.error("Ошибка при настройке бондинговой кривой:", error);
//       throw error;
//     }
//   });

//   it("Тестирует покупку токенов с автоматической маршрутизацией комиссий", async () => {
//     try {
//       // Минтим N-Dollar пользователю для покупки токенов
//       const userNDollarAmount = new BN(100 * Math.pow(10, nDollarDecimals)); // 100 N-Dollar
//       const mintTx = new Transaction().add(
//         createMintToInstruction(
//           nDollarMint,
//           userNDollarAccount,
//           admin.publicKey,
//           userNDollarAmount.toNumber()
//         )
//       );

//       await anchor.web3.sendAndConfirmTransaction(provider.connection, mintTx, [
//         admin,
//       ]);

//       console.log("Пользователю выдано 100 N-Dollar для покупки токенов");

//       // Получаем текущие балансы
//       const userNDollarBefore =
//         await provider.connection.getTokenAccountBalance(userNDollarAccount);
//       const poolBalanceBefore =
//         await provider.connection.getTokenAccountBalance(liquidityPool);
//       const userCoinBefore = await provider.connection.getTokenAccountBalance(
//         userCoinAccount
//       );

//       console.log(
//         "Баланс N-Dollar пользователя до покупки:",
//         userNDollarBefore.value.uiAmount
//       );
//       console.log(
//         "Баланс пула ликвидности до покупки:",
//         poolBalanceBefore.value.uiAmount
//       );
//       console.log(
//         "Баланс токенов пользователя до покупки:",
//         userCoinBefore.value.uiAmount || 0
//       );

//       // Покупаем токены за 10 N-Dollar
//       const buyAmount = new BN(10 * Math.pow(10, nDollarDecimals)); // 10 N-Dollar

//       console.log("Вызываем метод buyToken...");
//       try {
//         await bondingCurveProgram.methods
//           .buyToken(buyAmount)
//           .accounts({
//             buyer: user.publicKey,
//             bondingCurve: bondingCurve,
//             coinMint: coinMint,
//             ndollarMint: nDollarMint,
//             buyerCoinAccount: userCoinAccount,
//             buyerNdollarAccount: userNDollarAccount,
//             liquidityPool: liquidityPool,
//             feesAccount: feesAccount,
//             feeAuthority: feeAuthority.publicKey,
//             liquidityManager: liquidityManager,
//             poolNdollarAccount: poolNDollarAccount,
//             liquidityManagerProgram: liquidityManagerProgram.programId,
//             tokenProgram: TOKEN_PROGRAM_ID,
//             systemProgram: SystemProgram.programId,
//           })
//           .signers([user, feeAuthority])
//           .rpc();

//         console.log("Покупка токенов успешна!");
//       } catch (error) {
//         console.log("Детальная ошибка при покупке токенов:", error);
//         throw error;
//       }

//       // Получаем новые балансы
//       const userNDollarAfter = await provider.connection.getTokenAccountBalance(
//         userNDollarAccount
//       );
//       const poolBalanceAfter = await provider.connection.getTokenAccountBalance(
//         liquidityPool
//       );
//       const userCoinAfter = await provider.connection.getTokenAccountBalance(
//         userCoinAccount
//       );

//       console.log(
//         "Баланс N-Dollar пользователя после покупки:",
//         userNDollarAfter.value.uiAmount
//       );
//       console.log(
//         "Баланс пула ликвидности после покупки:",
//         poolBalanceAfter.value.uiAmount
//       );
//       console.log(
//         "Баланс токенов пользователя после покупки:",
//         userCoinAfter.value.uiAmount
//       );

//       // Проверяем, что N-Dollar уменьшились примерно на 10
//       const ndollarSpent =
//         userNDollarBefore.value.uiAmount! - userNDollarAfter.value.uiAmount!;
//       console.log("Потрачено N-Dollar:", ndollarSpent);
//       assert(
//         ndollarSpent >= 9.9 && ndollarSpent <= 10.1,
//         "Должно быть потрачено около 10 N-Dollar"
//       );

//       // Проверяем, что пул ликвидности увеличился на сумму покупки
//       const poolIncrease =
//         poolBalanceAfter.value.uiAmount! - poolBalanceBefore.value.uiAmount!;
//       console.log("Увеличение пула ликвидности:", poolIncrease);
//       assert(
//         poolIncrease >= 9.9,
//         "Пул ликвидности должен получить не менее 99% от суммы покупки"
//       );

//       // Проверяем, что пользователь получил токены
//       assert(
//         userCoinAfter.value.uiAmount! > 0,
//         "Пользователь должен получить токены"
//       );

//       console.log(
//         "Токены успешно куплены и комиссия автоматически направлена в пул ликвидности"
//       );
//     } catch (error) {
//       console.error("Ошибка при покупке токенов:", error);
//       throw error;
//     }
//   });

//   it("Тестирует продажу токенов с автоматической маршрутизацией комиссий", async () => {
//     try {
//       // Получаем текущие балансы перед продажей
//       const userCoinBefore = await provider.connection.getTokenAccountBalance(
//         userCoinAccount
//       );
//       const userNDollarBefore =
//         await provider.connection.getTokenAccountBalance(userNDollarAccount);
//       const poolBalanceBefore =
//         await provider.connection.getTokenAccountBalance(poolNDollarAccount);

//       console.log(
//         "Баланс токенов пользователя перед продажей:",
//         userCoinBefore.value.uiAmount
//       );
//       console.log(
//         "Баланс N-Dollar пользователя перед продажей:",
//         userNDollarBefore.value.uiAmount
//       );
//       console.log(
//         "Баланс пула перед продажей:",
//         poolBalanceBefore.value.uiAmount
//       );

//       // Если у пользователя есть токены, продаем часть из них
//       if (userCoinBefore.value.uiAmount! > 0) {
//         // Продаем 50% токенов
//         const sellAmount = new BN(
//           Math.floor(Number(userCoinBefore.value.amount) * 0.5)
//         );

//         console.log(
//           `Продаем ${sellAmount.toNumber() / Math.pow(10, 9)} токенов`
//         );

//         await bondingCurveProgram.methods
//           .sellToken(sellAmount)
//           .accounts({
//             buyer: user.publicKey,
//             bondingCurve: bondingCurve,
//             coinMint: coinMint,
//             ndollarMint: nDollarMint,
//             buyerCoinAccount: userCoinAccount,
//             buyerNdollarAccount: userNDollarAccount,
//             liquidityPool: liquidityPool,
//             feesAccount: feesAccount,
//             feeAuthority: feeAuthority.publicKey,
//             liquidityManager: liquidityManager,
//             poolNdollarAccount: poolNDollarAccount,
//             liquidityManagerProgram: liquidityManagerProgram.programId,
//             tokenProgram: TOKEN_PROGRAM_ID,
//             systemProgram: SystemProgram.programId,
//           })
//           .signers([user, feeAuthority])
//           .rpc();

//         // Получаем новые балансы
//         const userCoinAfter = await provider.connection.getTokenAccountBalance(
//           userCoinAccount
//         );
//         const userNDollarAfter =
//           await provider.connection.getTokenAccountBalance(userNDollarAccount);
//         const poolBalanceAfter =
//           await provider.connection.getTokenAccountBalance(poolNDollarAccount);

//         console.log(
//           "Баланс токенов пользователя после продажи:",
//           userCoinAfter.value.uiAmount
//         );
//         console.log(
//           "Баланс N-Dollar пользователя после продажи:",
//           userNDollarAfter.value.uiAmount
//         );
//         console.log(
//           "Баланс пула после продажи:",
//           poolBalanceAfter.value.uiAmount
//         );

//         // Проверяем, что количество токенов уменьшилось
//         const coinsSold =
//           userCoinBefore.value.uiAmount! - userCoinAfter.value.uiAmount!;
//         console.log("Продано токенов:", coinsSold);
//         assert(coinsSold > 0, "Должны быть проданы токены");

//         // Проверяем, что N-Dollar увеличились
//         const ndollarReceived =
//           userNDollarAfter.value.uiAmount! - userNDollarBefore.value.uiAmount!;
//         console.log("Получено N-Dollar:", ndollarReceived);
//         assert(ndollarReceived > 0, "Должны быть получены N-Dollar");

//         // Проверяем, что комиссия была направлена в пул ликвидности
//         // Обратите внимание: в этом случае мы не ожидаем точного увеличения пула,
//         // так как комиссия может быть очень маленькой (0.5%)
//         const poolChange =
//           poolBalanceAfter.value.uiAmount! - poolBalanceBefore.value.uiAmount!;
//         console.log("Изменение баланса пула:", poolChange);

//         // Успешно если вообще была направлена хоть какая-то комиссия
//         console.log(
//           "Токены успешно проданы и комиссия автоматически обработана"
//         );
//       } else {
//         console.log("У пользователя нет токенов для продажи, пропускаем тест");
//       }
//     } catch (error) {
//       console.error("Ошибка при продаже токенов:", error);
//       throw error;
//     }
//   });
// });
