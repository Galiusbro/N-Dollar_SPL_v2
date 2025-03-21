// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import { AdminControl } from "../target/types/admin_control";
// import { expect } from "chai";
// import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";

// describe("admin_control", () => {
//   // Настройка поставщика Anchor
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//   // Получение программы
//   const program = anchor.workspace.AdminControl as Program<AdminControl>;

//   // Создаем ключи для тестов
//   const adminAuthority = Keypair.generate();

//   // Другие программы, которые будут инициализированы в Admin Control
//   const bondingCurveProgram = Keypair.generate();
//   const genesisProgram = Keypair.generate();
//   const referralSystemProgram = Keypair.generate();
//   const tradingExchangeProgram = Keypair.generate();
//   const liquidityManagerProgram = Keypair.generate();
//   const ndollarMint = Keypair.generate();

//   // Дополнительные программы для тестирования авторизации
//   const externalProgram1 = Keypair.generate();
//   const externalProgram2 = Keypair.generate();

//   // PDA для Admin Config
//   let adminConfigPDA: PublicKey;

//   // Подготовка к тестам
//   before(async () => {
//     // Аирдроп SOL для тестового аккаунта
//     const signature = await provider.connection.requestAirdrop(
//       adminAuthority.publicKey,
//       10 * anchor.web3.LAMPORTS_PER_SOL // 10 SOL
//     );

//     // Ждем подтверждения транзакции
//     await provider.connection.confirmTransaction(signature);

//     // Находим PDA для admin_config
//     [adminConfigPDA] = PublicKey.findProgramAddressSync(
//       [Buffer.from("admin_config"), adminAuthority.publicKey.toBuffer()],
//       program.programId
//     );
//   });

//   it("Инициализирует Admin Config", async () => {
//     // Вызываем инструкцию initializeAdmin
//     await program.methods
//       .initializeAdmin()
//       .accounts({
//         authority: adminAuthority.publicKey,
//         adminConfig: adminConfigPDA,
//         systemProgram: SystemProgram.programId,
//       })
//       .signers([adminAuthority])
//       .rpc();

//     // Получаем данные Admin Config и проверяем их
//     const adminConfig = await program.account.adminConfig.fetch(adminConfigPDA);

//     // Проверяем, что аккаунт создан правильно
//     expect(adminConfig.authority.toString()).to.equal(
//       adminAuthority.publicKey.toString()
//     );
//     expect(adminConfig.version).to.equal(1); // Проверяем версию
//     expect(adminConfig.initializedModules).to.equal(0); // Изначально ни один модуль не инициализирован
//     expect(adminConfig.feeBasisPoints).to.equal(0); // Изначально комиссия равна 0
//   });

//   it("Инициализирует N-Dollar токен", async () => {
//     await program.methods
//       .initializeNdollar()
//       .accounts({
//         authority: adminAuthority.publicKey,
//         adminConfig: adminConfigPDA,
//         ndollarMint: ndollarMint.publicKey,
//       })
//       .signers([adminAuthority])
//       .rpc();

//     // Получаем обновленные данные Admin Config
//     const adminConfig = await program.account.adminConfig.fetch(adminConfigPDA);

//     // Проверяем, что N-Dollar модуль инициализирован
//     expect(adminConfig.ndollarMint.toString()).to.equal(
//       ndollarMint.publicKey.toString()
//     );
//     expect(adminConfig.initializedModules & 1).to.equal(1); // Бит для N-Dollar должен быть установлен
//   });

//   it("Инициализирует Bonding Curve программу", async () => {
//     await program.methods
//       .initializeBondingCurve()
//       .accounts({
//         authority: adminAuthority.publicKey,
//         adminConfig: adminConfigPDA,
//         bondingCurveProgram: bondingCurveProgram.publicKey,
//       })
//       .signers([adminAuthority])
//       .rpc();

//     // Получаем обновленные данные Admin Config
//     const adminConfig = await program.account.adminConfig.fetch(adminConfigPDA);

//     // Проверяем, что Bonding Curve программа инициализирована
//     expect(adminConfig.bondingCurveProgram.toString()).to.equal(
//       bondingCurveProgram.publicKey.toString()
//     );
//     expect(adminConfig.initializedModules & 2).to.equal(2); // Бит для Bonding Curve должен быть установлен
//   });

//   it("Инициализирует Genesis программу", async () => {
//     await program.methods
//       .initializeGenesis()
//       .accounts({
//         authority: adminAuthority.publicKey,
//         adminConfig: adminConfigPDA,
//         genesisProgram: genesisProgram.publicKey,
//       })
//       .signers([adminAuthority])
//       .rpc();

//     // Получаем обновленные данные Admin Config
//     const adminConfig = await program.account.adminConfig.fetch(adminConfigPDA);

//     // Проверяем, что Genesis программа инициализирована
//     expect(adminConfig.genesisProgram.toString()).to.equal(
//       genesisProgram.publicKey.toString()
//     );
//     expect(adminConfig.initializedModules & 4).to.equal(4); // Бит для Genesis должен быть установлен
//   });

//   it("Инициализирует Referral System программу", async () => {
//     await program.methods
//       .initializeReferralSystem()
//       .accounts({
//         authority: adminAuthority.publicKey,
//         adminConfig: adminConfigPDA,
//         referralSystemProgram: referralSystemProgram.publicKey,
//       })
//       .signers([adminAuthority])
//       .rpc();

//     // Получаем обновленные данные Admin Config
//     const adminConfig = await program.account.adminConfig.fetch(adminConfigPDA);

//     // Проверяем, что Referral System программа инициализирована
//     expect(adminConfig.referralSystemProgram.toString()).to.equal(
//       referralSystemProgram.publicKey.toString()
//     );
//     expect(adminConfig.initializedModules & 8).to.equal(8); // Бит для Referral System должен быть установлен
//   });

//   it("Инициализирует Trading Exchange программу", async () => {
//     await program.methods
//       .initializeTradingExchange()
//       .accounts({
//         authority: adminAuthority.publicKey,
//         adminConfig: adminConfigPDA,
//         tradingExchangeProgram: tradingExchangeProgram.publicKey,
//       })
//       .signers([adminAuthority])
//       .rpc();

//     // Получаем обновленные данные Admin Config
//     const adminConfig = await program.account.adminConfig.fetch(adminConfigPDA);

//     // Проверяем, что Trading Exchange программа инициализирована
//     expect(adminConfig.tradingExchangeProgram.toString()).to.equal(
//       tradingExchangeProgram.publicKey.toString()
//     );
//     expect(adminConfig.initializedModules & 16).to.equal(16); // Бит для Trading Exchange должен быть установлен
//   });

//   it("Инициализирует Liquidity Manager программу", async () => {
//     await program.methods
//       .initializeLiquidityManager()
//       .accounts({
//         authority: adminAuthority.publicKey,
//         adminConfig: adminConfigPDA,
//         liquidityManagerProgram: liquidityManagerProgram.publicKey,
//       })
//       .signers([adminAuthority])
//       .rpc();

//     // Получаем обновленные данные Admin Config
//     const adminConfig = await program.account.adminConfig.fetch(adminConfigPDA);

//     // Проверяем, что Liquidity Manager программа инициализирована
//     expect(adminConfig.liquidityManagerProgram.toString()).to.equal(
//       liquidityManagerProgram.publicKey.toString()
//     );
//     expect(adminConfig.initializedModules & 32).to.equal(32); // Бит для Liquidity Manager должен быть установлен
//   });

//   it("Авторизует внешнюю программу", async () => {
//     await program.methods
//       .authorizeProgram(externalProgram1.publicKey)
//       .accounts({
//         authority: adminAuthority.publicKey,
//         adminConfig: adminConfigPDA,
//       })
//       .signers([adminAuthority])
//       .rpc();

//     // Получаем обновленные данные Admin Config
//     const adminConfig = await program.account.adminConfig.fetch(adminConfigPDA);

//     // Проверяем, что программа добавлена в список авторизованных программ
//     expect(
//       adminConfig.authorizedPrograms.some(
//         (program) =>
//           program.toString() === externalProgram1.publicKey.toString()
//       )
//     ).to.be.true;
//   });

//   it("Авторизует вторую внешнюю программу", async () => {
//     await program.methods
//       .authorizeProgram(externalProgram2.publicKey)
//       .accounts({
//         authority: adminAuthority.publicKey,
//         adminConfig: adminConfigPDA,
//       })
//       .signers([adminAuthority])
//       .rpc();

//     // Получаем обновленные данные Admin Config
//     const adminConfig = await program.account.adminConfig.fetch(adminConfigPDA);

//     // Проверяем, что программа добавлена в список авторизованных программ
//     expect(
//       adminConfig.authorizedPrograms.some(
//         (program) =>
//           program.toString() === externalProgram2.publicKey.toString()
//       )
//     ).to.be.true;
//   });

//   it("Отзывает авторизацию у программы", async () => {
//     await program.methods
//       .revokeProgramAuthorization(externalProgram1.publicKey)
//       .accounts({
//         authority: adminAuthority.publicKey,
//         adminConfig: adminConfigPDA,
//       })
//       .signers([adminAuthority])
//       .rpc();

//     // Получаем обновленные данные Admin Config
//     const adminConfig = await program.account.adminConfig.fetch(adminConfigPDA);

//     // Проверяем, что программа удалена из списка авторизованных программ
//     expect(
//       adminConfig.authorizedPrograms.some(
//         (program) =>
//           program.toString() === externalProgram1.publicKey.toString()
//       )
//     ).to.be.false;

//     // Но вторая программа все еще должна быть в списке
//     expect(
//       adminConfig.authorizedPrograms.some(
//         (program) =>
//           program.toString() === externalProgram2.publicKey.toString()
//       )
//     ).to.be.true;
//   });

//   it("Обновляет комиссионные ставки", async () => {
//     const newFeeBasisPoints = 50; // 0.5%

//     await program.methods
//       .updateFees(newFeeBasisPoints)
//       .accounts({
//         authority: adminAuthority.publicKey,
//         adminConfig: adminConfigPDA,
//       })
//       .signers([adminAuthority])
//       .rpc();

//     // Получаем обновленные данные Admin Config
//     const adminConfig = await program.account.adminConfig.fetch(adminConfigPDA);

//     // Проверяем, что комиссия обновлена
//     expect(adminConfig.feeBasisPoints).to.equal(newFeeBasisPoints);
//   });

//   it("Не позволяет установить слишком высокую комиссию", async () => {
//     const tooHighFeeBasisPoints = 1500; // 15% > максимум 10%

//     try {
//       await program.methods
//         .updateFees(tooHighFeeBasisPoints)
//         .accounts({
//           authority: adminAuthority.publicKey,
//           adminConfig: adminConfigPDA,
//         })
//         .signers([adminAuthority])
//         .rpc();

//       // Если мы дошли до этой точки, значит ошибки не было, что неправильно
//       expect.fail("Должна быть ошибка: комиссия слишком высокая");
//     } catch (error) {
//       // Проверяем, что произошла ожидаемая ошибка
//       expect(error.message).to.include("Fee is too high");
//     }
//   });

//   it("Обрабатывает обновление структуры Admin Config", async () => {
//     await program.methods
//       .upgradeAdminConfig()
//       .accounts({
//         authority: adminAuthority.publicKey,
//         adminConfig: adminConfigPDA,
//       })
//       .signers([adminAuthority])
//       .rpc();

//     // Получаем обновленные данные Admin Config
//     const adminConfig = await program.account.adminConfig.fetch(adminConfigPDA);

//     // Предполагая, что версия увеличивается при обновлении
//     expect(adminConfig.version).to.be.above(1);
//   });

//   it("Не позволяет неавторизованному пользователю вызвать инструкции", async () => {
//     // Создаем другой аккаунт, не являющийся администратором
//     const nonAdminUser = Keypair.generate();

//     // Аирдроп SOL для нового аккаунта
//     const signature = await provider.connection.requestAirdrop(
//       nonAdminUser.publicKey,
//       1 * anchor.web3.LAMPORTS_PER_SOL
//     );

//     await provider.connection.confirmTransaction(signature);

//     try {
//       await program.methods
//         .updateFees(30)
//         .accounts({
//           authority: nonAdminUser.publicKey,
//           adminConfig: adminConfigPDA,
//         })
//         .signers([nonAdminUser])
//         .rpc();

//       // Если мы дошли до этой точки, значит ошибки не было, что неправильно
//       expect.fail("Должна быть ошибка: неавторизованный доступ");
//     } catch (error) {
//       // Проверяем, что произошла ожидаемая ошибка
//       expect(error.message).to.include("Unauthorized");
//     }
//   });
// });
