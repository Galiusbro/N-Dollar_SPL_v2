// // import * as anchor from "@coral-xyz/anchor";
// // import { Program } from "@coral-xyz/anchor";
// // import { AdminControl } from "../../target/types/admin_control";
// // import { PublicKey, Keypair } from "@solana/web3.js";
// // import { expect } from "chai";
// // import { createMint, getMint } from "@solana/spl-token";

// // describe("admin_control", () => {
// //   // Настройка провайдера Anchor
// //   const provider = anchor.AnchorProvider.env();
// //   anchor.setProvider(provider);

// //   // Программа admin_control
// //   const program = anchor.workspace.AdminControl as Program<AdminControl>;

// //   // Кошелек пользователя (админа)
// //   const authority = provider.wallet;

// //   // PDA для хранения конфигурации
// //   let adminConfigPda: PublicKey;
// //   let adminConfigBump: number;

// //   // Реальный mint для N-Dollar (вместо мнимого)
// //   let nDollarMint: PublicKey;

// //   // Мнимые Program ID для тестирования
// //   const mockBondingCurveProgram = Keypair.generate().publicKey;
// //   const mockGenesisProgram = Keypair.generate().publicKey;
// //   const mockReferralSystemProgram = Keypair.generate().publicKey;
// //   const mockTradingExchangeProgram = Keypair.generate().publicKey;
// //   const mockLiquidityManagerProgram = Keypair.generate().publicKey;

// //   before(async () => {
// //     // Получаем PDA для админской конфигурации
// //     [adminConfigPda, adminConfigBump] = PublicKey.findProgramAddressSync(
// //       [Buffer.from("admin_config"), authority.publicKey.toBuffer()],
// //       program.programId
// //     );

// //     console.log("Admin Config PDA:", adminConfigPda.toString());
// //     console.log("Admin Config Bump:", adminConfigBump);

// //     // Создаем mint для N-Dollar
// //     nDollarMint = await createMint(
// //       provider.connection,
// //       provider.wallet.payer, // Payer
// //       provider.wallet.publicKey, // Mint authority
// //       provider.wallet.publicKey, // Freeze authority
// //       9 // Decimals
// //     );

// //     console.log("N-Dollar Mint created:", nDollarMint.toString());
// //   });

// //   it("Initializes admin configuration", async () => {
// //     // Инициализируем админскую конфигурацию
// //     const tx = await program.methods
// //       .initializeAdmin()
// //       .accounts({
// //         authority: authority.publicKey,
// //         adminConfig: adminConfigPda,
// //         systemProgram: anchor.web3.SystemProgram.programId,
// //       })
// //       .rpc();

// //     console.log("Admin initialization tx:", tx);

// //     // Проверяем, что конфигурация создана правильно
// //     const adminConfig = await program.account.adminConfig.fetch(adminConfigPda);
// //     expect(adminConfig.authority.toString()).to.equal(
// //       authority.publicKey.toString()
// //     );
// //     expect(adminConfig.bump).to.equal(adminConfigBump);
// //     expect(adminConfig.initializedModules).to.equal(0);
// //     expect(adminConfig.feeBasisPoints).to.equal(30); // 0.3% по умолчанию
// //   });

// //   it("Initializes N-Dollar module", async () => {
// //     // Проверяем, что mint действительно существует
// //     const mintInfo = await getMint(provider.connection, nDollarMint);

// //     console.log("N-Dollar Mint info:", mintInfo.address.toString());

// //     // Инициализируем модуль N-Dollar
// //     const tx = await program.methods
// //       .initializeNdollar()
// //       .accounts({
// //         authority: authority.publicKey,
// //         adminConfig: adminConfigPda,
// //         ndollarMint: nDollarMint,
// //       })
// //       .rpc();

// //     console.log("N-Dollar initialization tx:", tx);

// //     // Проверяем, что N-Dollar инициализирован правильно
// //     const adminConfig = await program.account.adminConfig.fetch(adminConfigPda);
// //     expect(adminConfig.ndollarMint.toString()).to.equal(nDollarMint.toString());
// //     expect(adminConfig.initializedModules).to.equal(1); // Первый бит установлен
// //   });

// //   it("Initializes Bonding Curve module", async () => {
// //     // Инициализируем модуль Bonding Curve
// //     const tx = await program.methods
// //       .initializeBondingCurve()
// //       .accounts({
// //         authority: authority.publicKey,
// //         adminConfig: adminConfigPda,
// //         bondingCurveProgram: mockBondingCurveProgram,
// //       })
// //       .rpc();

// //     console.log("Bonding Curve initialization tx:", tx);

// //     // Проверяем, что Bonding Curve инициализирован правильно
// //     const adminConfig = await program.account.adminConfig.fetch(adminConfigPda);
// //     expect(adminConfig.bondingCurveProgram.toString()).to.equal(
// //       mockBondingCurveProgram.toString()
// //     );

// //     // Биты модулей: 1 (N-Dollar) + 2 (Bonding Curve) = 3
// //     expect(adminConfig.initializedModules).to.equal(3); // 00000011
// //   });

// //   it("Initializes Genesis module", async () => {
// //     // Инициализируем модуль Genesis
// //     const tx = await program.methods
// //       .initializeGenesis()
// //       .accounts({
// //         authority: authority.publicKey,
// //         adminConfig: adminConfigPda,
// //         genesisProgram: mockGenesisProgram,
// //       })
// //       .rpc();

// //     console.log("Genesis initialization tx:", tx);

// //     // Проверяем, что Genesis инициализирован правильно
// //     const adminConfig = await program.account.adminConfig.fetch(adminConfigPda);
// //     expect(adminConfig.genesisProgram.toString()).to.equal(
// //       mockGenesisProgram.toString()
// //     );

// //     // Биты модулей: 3 (N-Dollar + Bonding Curve) + 4 (Genesis) = 7
// //     expect(adminConfig.initializedModules).to.equal(7); // 00000111
// //   });

// //   it("Initializes Referral System module", async () => {
// //     // Инициализируем модуль Referral System
// //     const tx = await program.methods
// //       .initializeReferralSystem()
// //       .accounts({
// //         authority: authority.publicKey,
// //         adminConfig: adminConfigPda,
// //         referralSystemProgram: mockReferralSystemProgram,
// //       })
// //       .rpc();

// //     console.log("Referral System initialization tx:", tx);

// //     // Проверяем, что Referral System инициализирован правильно
// //     const adminConfig = await program.account.adminConfig.fetch(adminConfigPda);
// //     expect(adminConfig.referralSystemProgram.toString()).to.equal(
// //       mockReferralSystemProgram.toString()
// //     );

// //     // Биты модулей: 7 (N-Dollar + Bonding Curve + Genesis) + 8 (Referral) = 15
// //     expect(adminConfig.initializedModules).to.equal(15); // 00001111
// //   });

// //   it("Initializes Trading Exchange module", async () => {
// //     // Инициализируем модуль Trading Exchange
// //     const tx = await program.methods
// //       .initializeTradingExchange()
// //       .accounts({
// //         authority: authority.publicKey,
// //         adminConfig: adminConfigPda,
// //         tradingExchangeProgram: mockTradingExchangeProgram,
// //       })
// //       .rpc();

// //     console.log("Trading Exchange initialization tx:", tx);

// //     // Проверяем, что Trading Exchange инициализирован правильно
// //     const adminConfig = await program.account.adminConfig.fetch(adminConfigPda);
// //     expect(adminConfig.tradingExchangeProgram.toString()).to.equal(
// //       mockTradingExchangeProgram.toString()
// //     );

// //     // Биты модулей: 15 (предыдущие) + 16 (Trading Exchange) = 31
// //     expect(adminConfig.initializedModules).to.equal(31); // 00011111
// //   });

// //   it("Initializes Liquidity Manager module", async () => {
// //     // Инициализируем модуль Liquidity Manager
// //     const tx = await program.methods
// //       .initializeLiquidityManager()
// //       .accounts({
// //         authority: authority.publicKey,
// //         adminConfig: adminConfigPda,
// //         liquidityManagerProgram: mockLiquidityManagerProgram,
// //       })
// //       .rpc();

// //     console.log("Liquidity Manager initialization tx:", tx);

// //     // Проверяем, что Liquidity Manager инициализирован правильно
// //     const adminConfig = await program.account.adminConfig.fetch(adminConfigPda);
// //     expect(adminConfig.liquidityManagerProgram.toString()).to.equal(
// //       mockLiquidityManagerProgram.toString()
// //     );

// //     // Биты модулей: 31 (предыдущие) + 32 (Liquidity Manager) = 63
// //     expect(adminConfig.initializedModules).to.equal(63); // 00111111
// //   });

// //   it("Updates fees", async () => {
// //     // Обновляем комиссию до 0.5%
// //     const newFee = 50; // 0.5%
// //     const tx = await program.methods
// //       .updateFees(newFee)
// //       .accounts({
// //         authority: authority.publicKey,
// //         adminConfig: adminConfigPda,
// //       })
// //       .rpc();

// //     console.log("Fee update tx:", tx);

// //     // Проверяем, что комиссия обновлена правильно
// //     const adminConfig = await program.account.adminConfig.fetch(adminConfigPda);
// //     expect(adminConfig.feeBasisPoints).to.equal(newFee);
// //   });

// //   it("Authorizes and revokes programs", async () => {
// //     // Тестовая программа для авторизации
// //     const testProgram = Keypair.generate().publicKey;

// //     // Авторизуем программу
// //     const authTx = await program.methods
// //       .authorizeProgram(testProgram)
// //       .accounts({
// //         authority: authority.publicKey,
// //         adminConfig: adminConfigPda,
// //       })
// //       .rpc();

// //     console.log("Program authorization tx:", authTx);

// //     // Проверяем, что программа авторизована
// //     let adminConfig = await program.account.adminConfig.fetch(adminConfigPda);
// //     expect(adminConfig.authorizedPrograms[0].toString()).to.equal(
// //       testProgram.toString()
// //     );

// //     // Отзываем авторизацию
// //     const revokeTx = await program.methods
// //       .revokeProgramAuthorization(testProgram)
// //       .accounts({
// //         authority: authority.publicKey,
// //         adminConfig: adminConfigPda,
// //       })
// //       .rpc();

// //     console.log("Program revocation tx:", revokeTx);

// //     // Проверяем, что авторизация отозвана
// //     adminConfig = await program.account.adminConfig.fetch(adminConfigPda);
// //     expect(adminConfig.authorizedPrograms[0].toString()).to.not.equal(
// //       testProgram.toString()
// //     );
// //     expect(adminConfig.authorizedPrograms[0].toString()).to.equal(
// //       PublicKey.default.toString()
// //     );
// //   });

// //   it("Only authorized admin can execute admin functions", async () => {
// //     // Создаем неавторизованного пользователя
// //     const unauthorizedUser = Keypair.generate();

// //     // Попытка обновить комиссию неавторизованным пользователем должна завершиться ошибкой
// //     try {
// //       const tx = await program.methods
// //         .updateFees(100)
// //         .accounts({
// //           authority: unauthorizedUser.publicKey,
// //           adminConfig: adminConfigPda,
// //         })
// //         .rpc();

// //       // Если выполнение дошло до этой точки, значит ошибки не было - это неправильно
// //       expect.fail("Транзакция должна была завершиться ошибкой");
// //     } catch (error) {
// //       // Ожидаем ошибку, проверяем что ошибка связана с правами доступа
// //       expect(error.toString()).to.include("Error");
// //       console.log(
// //         "Получена ожидаемая ошибка при попытке неавторизованного доступа:",
// //         error.toString().substring(0, 150) + "..."
// //       );
// //     }

// //     // Проверяем, что значение feeBasisPoints не изменилось
// //     const adminConfig = await program.account.adminConfig.fetch(adminConfigPda);
// //     expect(adminConfig.feeBasisPoints).to.equal(50); // Значение, установленное в предыдущем тесте
// //   });

// //   it("Upgrades admin configuration version or handles 'already upgraded' error", async () => {
// //     // Получаем текущую версию конфигурации
// //     let adminConfig = await program.account.adminConfig.fetch(adminConfigPda);
// //     const currentVersion = adminConfig.version;

// //     console.log("Current admin config version:", currentVersion);

// //     try {
// //       // Пытаемся обновить конфигурацию
// //       const tx = await program.methods
// //         .upgradeAdminConfig()
// //         .accounts({
// //           authority: authority.publicKey,
// //           adminConfig: adminConfigPda,
// //         })
// //         .rpc();

// //       console.log("Config upgrade tx:", tx);

// //       // Проверяем, что версия обновилась
// //       adminConfig = await program.account.adminConfig.fetch(adminConfigPda);
// //       expect(adminConfig.version).to.be.greaterThan(currentVersion);
// //       console.log(
// //         `Config version updated: ${currentVersion} -> ${adminConfig.version}`
// //       );
// //     } catch (error) {
// //       // Проверяем, что ошибка связана с тем, что конфигурация уже обновлена
// //       if (error.toString().includes("AlreadyUpgraded")) {
// //         console.log(
// //           "Admin Config is already on the latest version, as expected:",
// //           currentVersion
// //         );
// //       } else {
// //         // Если ошибка другая, выбрасываем её дальше
// //         throw error;
// //       }
// //     }
// //   });

// //   // Функция смены администратора еще не реализована в программе admin_control
// //   // TODO: Добавить тест для смены администратора, когда функция будет реализована
// // });

// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import { NDollarToken } from "../../target/types/n_dollar_token";
// import {
//   PublicKey,
//   Keypair,
//   SystemProgram,
//   SYSVAR_RENT_PUBKEY,
// } from "@solana/web3.js";
// import {
//   TOKEN_PROGRAM_ID,
//   createMint,
//   getAssociatedTokenAddress,
// } from "@solana/spl-token";
// import { assert } from "chai";

// // import { AdminControl } from "../../target/types/admin_control";
// // import { PublicKey, Keypair } from "@solana/web3.js";
// // import { expect } from "chai";
// // import { createMint, getMint } from "@solana/spl-token";

// describe("n-dollar-token", () => {
//   // Configure the client to use the local cluster
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//   const program = anchor.workspace.NDollarToken as Program<NDollarToken>;
//   const wallet = provider.wallet as anchor.Wallet;

//   // Set up test variables
//   let mint: Keypair;
//   let adminAccount: PublicKey;
//   let adminAccountBump: number;
//   let tokenAccount: PublicKey;
//   let metadata: PublicKey;

//   // Metaplex Metadata Program ID
//   const TOKEN_METADATA_PROGRAM_ID = new PublicKey(
//     "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
//   );

//   before(async () => {
//     // Create a new keypair for the mint
//     mint = Keypair.generate();

//     // Find the PDA for the admin account
//     const [adminAccountPDA, bump] = await PublicKey.findProgramAddress(
//       [Buffer.from("admin_account"), mint.publicKey.toBuffer()],
//       program.programId
//     );

//     adminAccount = adminAccountPDA;
//     adminAccountBump = bump;

//     // Derive the metadata account address (same way as Metaplex)
//     const [metadataPDA] = await PublicKey.findProgramAddress(
//       [
//         Buffer.from("metadata"),
//         TOKEN_METADATA_PROGRAM_ID.toBuffer(),
//         mint.publicKey.toBuffer(),
//       ],
//       TOKEN_METADATA_PROGRAM_ID
//     );

//     metadata = metadataPDA;

//     // Prepare the associated token account for the wallet
//     tokenAccount = await getAssociatedTokenAddress(
//       mint.publicKey,
//       wallet.publicKey
//     );
//   });

//   it("Initializes N-Dollar token", async () => {
//     try {
//       // Используем правильный ID метаданных (такой же, как ожидается в контракте)
//       const metaplexProgramId = new PublicKey(
//         "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
//       );

//       await program.methods
//         .initializeNDollar(
//           "N-Dollar",
//           "NDOL",
//           "https://token-metadata-uri.com",
//           9
//         )
//         .accounts({
//           authority: wallet.publicKey,
//           mint: mint.publicKey,
//           metadata: metadata,
//           adminAccount: adminAccount,
//           adminConfig: null, // Optional, pass null if not used
//           adminControlProgram: null, // Optional, pass null if not used
//           systemProgram: SystemProgram.programId,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           metadataProgram: metaplexProgramId,
//           rent: SYSVAR_RENT_PUBKEY,
//         })
//         .signers([mint])
//         .rpc();

//       console.log("✅ N-Dollar Token initialized successfully");

//       // Verify the admin account was created
//       const adminAccountData = await program.account.adminAccount.fetch(
//         adminAccount
//       );
//       assert.equal(
//         adminAccountData.authority.toString(),
//         wallet.publicKey.toString()
//       );
//       assert.equal(adminAccountData.mint.toString(), mint.publicKey.toString());
//     } catch (error) {
//       console.error("❌ Error initializing N-Dollar token:", error);
//       throw error;
//     }
//   });

//   // Additional tests for other functionality
//   it("Can mint tokens with proper authorization", async () => {
//     // Test mint_supply functionality
//   });

//   it("Can burn tokens with proper authorization", async () => {
//     // Test burn_tokens functionality
//   });
// });
