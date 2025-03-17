// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
// import {
//   TOKEN_PROGRAM_ID,
//   getAssociatedTokenAddress,
//   createAssociatedTokenAccountInstruction,
// } from "@solana/spl-token";
// import { assert } from "chai";
// import BN from "bn.js";
// import { getMetadataUrl } from "./utils/metadata";

// // Константа для программы метаданных Metaplex
// const METADATA_PROGRAM_ID = new PublicKey(
//   "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
// );

// describe("N-Dollar Exchange & Coin Creation Platform на devnet", () => {
//   // Настройка провайдера для devnet
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//   // Загрузка программ
//   const nDollarProgram = anchor.workspace.NDollarToken as Program;
//   const genesisProgram = anchor.workspace.Genesis as Program;
//   const bondingCurveProgram = anchor.workspace.BondingCurve as Program;
//   const liquidityManagerProgram = anchor.workspace.LiquidityManager as Program;
//   const tradingExchangeProgram = anchor.workspace.TradingExchange as Program;
//   const referralSystemProgram = anchor.workspace.ReferralSystem as Program;

//   // Кошельки для тестирования
//   const admin = Keypair.generate();
//   const user1 = Keypair.generate();

//   // Переменные для хранения аккаунтов
//   let nDollarMint: PublicKey;
//   let adminNDollarAccount: PublicKey;
//   let adminAccount: PublicKey;
//   let metadataAccount: PublicKey;

//   // Параметры для инициализации N-Dollar
//   const nDollarName = "N-Dollar";
//   const nDollarSymbol = "NDOL";
//   const nDollarDecimals = 9;
//   let nDollarUri: string; // Будет установлен позже

//   it("Инициализирует тестовые аккаунты на devnet", async () => {
//     // Выделяем SOL для тестовых аккаунтов на devnet
//     // Мы используем airdrops, но на devnet они могут быть ограничены
//     try {
//       await provider.connection.requestAirdrop(
//         admin.publicKey,
//         1 * anchor.web3.LAMPORTS_PER_SOL
//       );
//       console.log(
//         `Запрошен airdrop SOL для admin: ${admin.publicKey.toString()}`
//       );
//     } catch (error) {
//       console.error("Ошибка при запросе airdrop:", error);
//       // Если airdrop не сработал, тест может не удасться
//       // В реальном сценарии нужно предварительно пополнить кошелек
//     }

//     // Ждем подтверждения транзакций
//     await new Promise((resolve) => setTimeout(resolve, 5000));

//     // Получаем баланс
//     const balance = await provider.connection.getBalance(admin.publicKey);
//     console.log(`Баланс admin: ${balance / anchor.web3.LAMPORTS_PER_SOL} SOL`);
//   });

//   it("Создает метаданные для N-Dollar токена", async () => {
//     try {
//       // Генерируем URI для метаданных
//       nDollarUri = await getMetadataUrl(
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

//   it("Инициализирует N-Dollar токен с метаданными", async () => {
//     // Пропускаем тест, если у админа недостаточно SOL
//     const balance = await provider.connection.getBalance(admin.publicKey);
//     if (balance < 0.5 * anchor.web3.LAMPORTS_PER_SOL) {
//       console.log(
//         "Пропускаем тест - недостаточно SOL для инициализации токена"
//       );
//       return;
//     }

//     // Создаем минт для N-Dollar
//     const mintKeypair = Keypair.generate();
//     nDollarMint = mintKeypair.publicKey;
//     console.log("Создан минт N-Dollar:", nDollarMint.toString());

//     // Находим PDA для admin аккаунта
//     const [adminAccountPDA] = await PublicKey.findProgramAddress(
//       [Buffer.from("admin_account"), nDollarMint.toBuffer()],
//       nDollarProgram.programId
//     );
//     adminAccount = adminAccountPDA;

//     // Находим аккаунт метаданных
//     const [metadataAccountPDA] = await PublicKey.findProgramAddress(
//       [
//         Buffer.from("metadata"),
//         METADATA_PROGRAM_ID.toBuffer(),
//         nDollarMint.toBuffer(),
//       ],
//       METADATA_PROGRAM_ID
//     );
//     metadataAccount = metadataAccountPDA;

//     // Вычисляем адрес ассоциированного токен-аккаунта для админа
//     adminNDollarAccount = await getAssociatedTokenAddress(
//       nDollarMint,
//       admin.publicKey
//     );

//     try {
//       // Инициализируем N-Dollar токен с метаданными
//       const tx = await nDollarProgram.methods
//         .initializeNDollar(
//           nDollarName,
//           nDollarSymbol,
//           nDollarUri,
//           nDollarDecimals
//         )
//         .accounts({
//           authority: admin.publicKey,
//           mint: nDollarMint,
//           metadata: metadataAccount,
//           adminAccount: adminAccount,
//           systemProgram: SystemProgram.programId,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           metadataProgram: METADATA_PROGRAM_ID,
//           rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//         })
//         .signers([admin, mintKeypair])
//         .rpc();

//       console.log("N-Dollar токен успешно инициализирован, транзакция:", tx);
//       console.log("Mint:", nDollarMint.toString());
//       console.log("Admin Account:", adminAccount.toString());
//       console.log("Metadata Account:", metadataAccount.toString());

//       // Создаем ассоциированный токен-аккаунт для администратора
//       const createAtaTx = await provider.sendAndConfirm(
//         new anchor.web3.Transaction().add(
//           createAssociatedTokenAccountInstruction(
//             admin.publicKey, // payer
//             adminNDollarAccount, // associatedToken
//             admin.publicKey, // owner
//             nDollarMint // mint
//           )
//         ),
//         [admin]
//       );

//       console.log(
//         "Создан ассоциированный токен-аккаунт для администратора:",
//         adminNDollarAccount.toString()
//       );
//     } catch (error) {
//       console.error("Ошибка при инициализации N-Dollar:", error);

//       // Показываем более подробную информацию об ошибке
//       if (error.logs) {
//         console.error("Логи ошибки:", error.logs);
//       }

//       throw error;
//     }
//   });

//   it("Минтит N-Dollar токены", async () => {
//     // Пропускаем тест, если предыдущие тесты не прошли
//     if (!nDollarMint || !adminNDollarAccount || !adminAccount) {
//       console.log("Пропускаем тест - токен не был инициализирован");
//       return;
//     }

//     try {
//       const tx = await nDollarProgram.methods
//         .mintSupply(new BN(1000000000)) // 1 миллиард токенов (с учетом деленности)
//         .accounts({
//           authority: admin.publicKey,
//           mint: nDollarMint,
//           adminAccount: adminAccount,
//           tokenAccount: adminNDollarAccount,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           systemProgram: SystemProgram.programId,
//         })
//         .signers([admin])
//         .rpc();

//       console.log("N-Dollar токены успешно минтнуты, транзакция:", tx);

//       // Проверяем баланс токенов
//       const tokenBalance = await provider.connection.getTokenAccountBalance(
//         adminNDollarAccount
//       );
//       console.log(
//         `Баланс N-Dollar администратора: ${tokenBalance.value.uiAmount}`
//       );
//     } catch (error) {
//       console.error("Ошибка при минтинге N-Dollar:", error);

//       // Показываем более подробную информацию об ошибке
//       if (error.logs) {
//         console.error("Логи ошибки:", error.logs);
//       }

//       throw error;
//     }
//   });
// });
