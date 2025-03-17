// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import {
//   PublicKey,
//   Keypair,
//   SystemProgram,
//   Transaction,
// } from "@solana/web3.js";
// import {
//   TOKEN_PROGRAM_ID,
//   getAssociatedTokenAddress,
//   createAssociatedTokenAccountInstruction,
//   createInitializeMintInstruction,
//   createMintToInstruction,
//   getMint,
// } from "@solana/spl-token";
// import { assert } from "chai";
// import BN from "bn.js";
// import { createTokenMetadata } from "./utils/metadata";

// // Константа для программы метаданных Metaplex
// const METADATA_PROGRAM_ID = new PublicKey(
//   "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
// );

// describe("N-Dollar Exchange & Coin Creation Platform с моками", () => {
//   // Настройка провайдера (используем localhost)
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//   // Кошельки для тестирования
//   const admin = Keypair.generate();
//   const user1 = Keypair.generate();

//   // Переменные для хранения аккаунтов
//   let nDollarMint: PublicKey;
//   let adminNDollarAccount: PublicKey;

//   // Параметры для инициализации N-Dollar
//   const nDollarName = "N-Dollar";
//   const nDollarSymbol = "NDOL";
//   const nDollarDecimals = 9;
//   let nDollarUri: string; // Будет установлен позже

//   it("Инициализирует тестовые аккаунты", async () => {
//     // Выделяем SOL для тестовых аккаунтов
//     try {
//       await provider.connection.requestAirdrop(
//         admin.publicKey,
//         10 * anchor.web3.LAMPORTS_PER_SOL
//       );
//       console.log(
//         `Запрошен airdrop SOL для admin: ${admin.publicKey.toString()}`
//       );
//     } catch (error) {
//       console.error("Ошибка при запросе airdrop:", error);
//     }

//     // Ждем подтверждения транзакций
//     await new Promise((resolve) => setTimeout(resolve, 2000));

//     // Получаем баланс
//     const balance = await provider.connection.getBalance(admin.publicKey);
//     console.log(`Баланс admin: ${balance / anchor.web3.LAMPORTS_PER_SOL} SOL`);
//   });

//   it("Создает метаданные для N-Dollar токена", async () => {
//     try {
//       // Генерируем URI для метаданных
//       nDollarUri = await createTokenMetadata(
//         nDollarName,
//         nDollarSymbol,
//         "Стейблкоин N-Dollar для децентрализованной биржи"
//       );
//       console.log(`Создан URI для метаданных: ${nDollarUri}`);
//     } catch (error) {
//       console.error("Ошибка при создании метаданных:", error);
//       throw error;
//     }
//   });

//   it("Создает N-Dollar токен (без метаданных)", async () => {
//     // Создаем минт для N-Dollar
//     const mintKeypair = Keypair.generate();
//     nDollarMint = mintKeypair.publicKey;
//     console.log("Создан минт N-Dollar:", nDollarMint.toString());

//     try {
//       // Создаем минт напрямую без вызова программы метаданных
//       const lamports =
//         await provider.connection.getMinimumBalanceForRentExemption(82);

//       const initMintTx = new Transaction().add(
//         SystemProgram.createAccount({
//           fromPubkey: admin.publicKey,
//           newAccountPubkey: nDollarMint,
//           space: 82, // Размер для токен-минта
//           lamports: lamports,
//           programId: TOKEN_PROGRAM_ID,
//         }),
//         createInitializeMintInstruction(
//           nDollarMint,
//           nDollarDecimals,
//           admin.publicKey,
//           admin.publicKey
//         )
//       );

//       // Отправляем транзакцию
//       await provider.sendAndConfirm(initMintTx, [admin, mintKeypair]);
//       console.log("Минт успешно инициализирован");

//       // Вычисляем адрес ассоциированного токен-аккаунта для админа
//       adminNDollarAccount = await getAssociatedTokenAddress(
//         nDollarMint,
//         admin.publicKey
//       );

//       // Создаем ассоциированный токен-аккаунт для администратора
//       const createAtaTx = new Transaction().add(
//         createAssociatedTokenAccountInstruction(
//           admin.publicKey, // payer
//           adminNDollarAccount, // associatedToken
//           admin.publicKey, // owner
//           nDollarMint // mint
//         )
//       );

//       await provider.sendAndConfirm(createAtaTx, [admin]);
//       console.log(
//         "Создан ассоциированный токен-аккаунт для администратора:",
//         adminNDollarAccount.toString()
//       );

//       // Симулируем успешное создание метаданных
//       console.log(
//         `Симуляция: Созданы метаданные токена N-Dollar с URI: ${nDollarUri}`
//       );
//     } catch (error) {
//       console.error("Ошибка при создании токена:", error);
//       throw error;
//     }
//   });

//   it("Минтит N-Dollar токены", async () => {
//     try {
//       // Минтим токены напрямую
//       const mintTx = new Transaction().add(
//         createMintToInstruction(
//           nDollarMint,
//           adminNDollarAccount,
//           admin.publicKey,
//           1_000_000_000_000 // 1 миллион токенов с учетом 9 десятичных знаков
//         )
//       );

//       await provider.sendAndConfirm(mintTx, [admin]);
//       console.log("Токены успешно минтнуты!");

//       // Проверяем баланс токенов
//       const tokenBalance = await provider.connection.getTokenAccountBalance(
//         adminNDollarAccount
//       );
//       console.log(
//         `Баланс N-Dollar администратора: ${tokenBalance.value.uiAmount}`
//       );

//       // Проверяем, что минт успешно создан и имеет правильную decimal places
//       const mintInfo = await getMint(provider.connection, nDollarMint);
//       assert.equal(
//         mintInfo.decimals,
//         nDollarDecimals,
//         "Decimals должны соответствовать заданным"
//       );
//     } catch (error) {
//       console.error("Ошибка при минтинге токенов:", error);
//       throw error;
//     }
//   });
// });
