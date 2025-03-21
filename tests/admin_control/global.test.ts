// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import { AdminControl } from "../../target/types/admin_control";
// import { PublicKey, Keypair } from "@solana/web3.js";
// import { expect } from "chai";
// import { createMint, getMint } from "@solana/spl-token";
// import { createAssociatedTokenAccount } from "@solana/spl-token";

// describe("admin_control", () => {
//   // Настройка провайдера Anchor
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//   // Программа admin_control
//   const program = anchor.workspace.AdminControl as Program<AdminControl>;

//   // Кошелек пользователя (админа)
//   const authority = provider.wallet;

//   // PDA для хранения конфигурации
//   let adminConfigPda: PublicKey;
//   let adminConfigBump: number;

//   // Реальный mint для N-Dollar (вместо мнимого)
//   let nDollarMint: PublicKey;

//   // Мнимые Program ID для тестирования
//   const mockBondingCurveProgram = Keypair.generate().publicKey;
//   const mockGenesisProgram = Keypair.generate().publicKey;
//   const mockReferralSystemProgram = Keypair.generate().publicKey;
//   const mockTradingExchangeProgram = Keypair.generate().publicKey;
//   const mockLiquidityManagerProgram = Keypair.generate().publicKey;

//   before(async () => {
//     // Получаем PDA для админской конфигурации
//     [adminConfigPda, adminConfigBump] = PublicKey.findProgramAddressSync(
//       [Buffer.from("admin_config"), authority.publicKey.toBytes()],
//       program.programId
//     );

//     console.log("Admin Config PDA:", adminConfigPda.toString());
//     console.log("Admin Config Bump:", adminConfigBump);

//     // Создаем mint для N-Dollar
//     nDollarMint = await createMint(
//       provider.connection,
//       provider.wallet.payer, // Payer
//       provider.wallet.publicKey, // Mint authority
//       provider.wallet.publicKey, // Freeze authority
//       9 // Decimals
//     );

//     console.log("N-Dollar Mint created:", nDollarMint.toString());
//   });

//   it("Initializes admin configuration", async () => {
//     // Инициализируем админскую конфигурацию
//     const tx = await program.methods
//       .initializeAdmin()
//       .accounts({
//         authority: authority.publicKey,
//         adminConfig: adminConfigPda,
//         systemProgram: anchor.web3.SystemProgram.programId,
//       })
//       .rpc();

//     console.log("Admin initialization tx:", tx);

//     // Проверяем, что конфигурация создана правильно
//     const adminConfig = await program.account.adminConfig.fetch(adminConfigPda);
//     expect(adminConfig.authority.toString()).to.equal(
//       authority.publicKey.toString()
//     );
//     expect(adminConfig.bump).to.equal(adminConfigBump);
//     expect(adminConfig.initializedModules).to.equal(0);
//     expect(adminConfig.feeBasisPoints).to.equal(30); // 0.3% по умолчанию
//   });

//   it("Initializes N-Dollar module", async () => {
//     // Проверяем, что mint действительно существует
//     const mintInfo = await getMint(provider.connection, nDollarMint);

//     console.log("N-Dollar Mint info:", mintInfo.address.toString());

//     // Инициализируем модуль N-Dollar
//     const tx = await program.methods
//       .initializeNdollar()
//       .accounts({
//         authority: authority.publicKey,
//         adminConfig: adminConfigPda,
//         ndollarMint: nDollarMint,
//       })
//       .rpc();

//     console.log("N-Dollar initialization tx:", tx);

//     // Проверяем, что N-Dollar инициализирован правильно
//     const adminConfig = await program.account.adminConfig.fetch(adminConfigPda);
//     expect(adminConfig.ndollarMint.toString()).to.equal(nDollarMint.toString());
//     expect(adminConfig.initializedModules).to.equal(1); // Первый бит установлен
//   });

//   it("Initializes Bonding Curve module", async () => {
//     // Инициализируем модуль Bonding Curve
//     const tx = await program.methods
//       .initializeBondingCurve()
//       .accounts({
//         authority: authority.publicKey,
//         adminConfig: adminConfigPda,
//         bondingCurveProgram: mockBondingCurveProgram,
//       })
//       .rpc();

//     console.log("Bonding Curve initialization tx:", tx);

//     // Проверяем, что Bonding Curve инициализирован правильно
//     const adminConfig = await program.account.adminConfig.fetch(adminConfigPda);
//     expect(adminConfig.bondingCurveProgram.toString()).to.equal(
//       mockBondingCurveProgram.toString()
//     );

//     // Биты модулей: 1 (N-Dollar) + 2 (Bonding Curve) = 3
//     expect(adminConfig.initializedModules).to.equal(3); // 00000011
//   });

//   it("Updates fees", async () => {
//     // Обновляем комиссию до 0.5%
//     const newFee = 50; // 0.5%
//     const tx = await program.methods
//       .updateFees(newFee)
//       .accounts({
//         authority: authority.publicKey,
//         adminConfig: adminConfigPda,
//       })
//       .rpc();

//     console.log("Fee update tx:", tx);

//     // Проверяем, что комиссия обновлена правильно
//     const adminConfig = await program.account.adminConfig.fetch(adminConfigPda);
//     expect(adminConfig.feeBasisPoints).to.equal(newFee);
//   });

//   it("Authorizes and revokes programs", async () => {
//     // Тестовая программа для авторизации
//     const testProgram = Keypair.generate().publicKey;

//     // Авторизуем программу
//     const authTx = await program.methods
//       .authorizeProgram(testProgram)
//       .accounts({
//         authority: authority.publicKey,
//         adminConfig: adminConfigPda,
//       })
//       .rpc();

//     console.log("Program authorization tx:", authTx);

//     // Проверяем, что программа авторизована
//     let adminConfig = await program.account.adminConfig.fetch(adminConfigPda);
//     expect(adminConfig.authorizedPrograms[0].toString()).to.equal(
//       testProgram.toString()
//     );

//     // Отзываем авторизацию
//     const revokeTx = await program.methods
//       .revokeProgramAuthorization(testProgram)
//       .accounts({
//         authority: authority.publicKey,
//         adminConfig: adminConfigPda,
//       })
//       .rpc();

//     console.log("Program revocation tx:", revokeTx);

//     // Проверяем, что авторизация отозвана
//     adminConfig = await program.account.adminConfig.fetch(adminConfigPda);
//     expect(adminConfig.authorizedPrograms[0].toString()).to.not.equal(
//       testProgram.toString()
//     );
//     expect(adminConfig.authorizedPrograms[0].toString()).to.equal(
//       PublicKey.default.toString()
//     );
//   });

//   // ============ ТЕСТЫ ДЛЯ N-DOLLAR TOKEN ============
//   describe("n-dollar-token tests", () => {
//     // PDA для admin_account N-Dollar
//     let nDollarAdminAccount: PublicKey;
//     let nDollarAdminAccountBump: number;

//     // Токен-аккаунт для админа
//     let adminTokenAccount: PublicKey;

//     // Токен-аккаунт для пула ликвидности
//     let liquidityPoolAccount: PublicKey;

//     // Программа N-Dollar Token
//     const nDollarTokenProgram = anchor.workspace.NDollarToken;

//     before(async () => {
//       // Получаем PDA для admin_account
//       [nDollarAdminAccount, nDollarAdminAccountBump] =
//         PublicKey.findProgramAddressSync(
//           [Buffer.from("admin_account"), nDollarMint.toBytes()],
//           nDollarTokenProgram.programId
//         );

//       console.log(
//         "N-Dollar Admin Account PDA:",
//         nDollarAdminAccount.toString()
//       );
//       console.log("N-Dollar Admin Account Bump:", nDollarAdminAccountBump);

//       // Создаем токен-аккаунт для администратора
//       adminTokenAccount = await createAssociatedTokenAccount(
//         provider.connection,
//         provider.wallet.payer,
//         nDollarMint,
//         provider.wallet.publicKey
//       );

//       console.log("Admin Token Account:", adminTokenAccount.toString());

//       // Создаем токен-аккаунт для пула ликвидности (для тестирования используем отдельный аккаунт)
//       const liquidityKeypair = Keypair.generate();
//       liquidityPoolAccount = await createAssociatedTokenAccount(
//         provider.connection,
//         provider.wallet.payer,
//         nDollarMint,
//         liquidityKeypair.publicKey
//       );

//       console.log(
//         "Liquidity Pool Token Account:",
//         liquidityPoolAccount.toString()
//       );
//     });

//     it("Initializes N-Dollar token", async () => {
//       // Инициализируем токен N-Dollar
//       const tx = await nDollarTokenProgram.methods
//         .initializeNDollar(
//           "N-Dollar", // name
//           "NDOL", // symbol
//           "https://n-dollar.com/metadata.json", // uri
//           9 // decimals
//         )
//         .accounts({
//           authority: authority.publicKey,
//           mint: nDollarMint,
//           adminAccount: nDollarAdminAccount,
//           adminTokenAccount: adminTokenAccount,
//           liquidityPoolAccount: liquidityPoolAccount,
//           adminConfig: adminConfigPda,
//           adminControlProgram: program.programId,
//           systemProgram: anchor.web3.SystemProgram.programId,
//           tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
//           rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//         })
//         .signers([provider.wallet.payer])
//         .rpc();

//       console.log("N-Dollar token initialization tx:", tx);

//       // Проверяем, что admin_account создан правильно
//       const adminAccount = await nDollarTokenProgram.account.adminAccount.fetch(
//         nDollarAdminAccount
//       );
//       expect(adminAccount.authority.toString()).to.equal(
//         authority.publicKey.toString()
//       );
//       expect(adminAccount.mint.toString()).to.equal(nDollarMint.toString());
//       expect(adminAccount.bump).to.equal(nDollarAdminAccountBump);
//       expect(adminAccount.currentMintWeek).to.equal(1); // Начинаем с первой недели
//     });

//     it("Mints supply according to schedule", async () => {
//       // Получаем баланс до минта
//       const balanceBefore = await provider.connection.getTokenAccountBalance(
//         adminTokenAccount
//       );
//       console.log(
//         "Admin token balance before mint:",
//         balanceBefore.value.uiAmount
//       );

//       // Выполняем минтинг токенов согласно расписанию
//       const mintAmount = 1_000_000 * 1_000_000_000; // 1 млн токенов с учетом decimals
//       const tx = await nDollarTokenProgram.methods
//         .mintSupply(new anchor.BN(mintAmount))
//         .accounts({
//           authority: authority.publicKey,
//           mint: nDollarMint,
//           adminAccount: nDollarAdminAccount,
//           tokenAccount: adminTokenAccount,
//           liquidityPoolAccount: liquidityPoolAccount,
//           tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
//           systemProgram: anchor.web3.SystemProgram.programId,
//         })
//         .rpc();

//       console.log("Mint supply tx:", tx);

//       // Проверяем, что баланс увеличился
//       const balanceAfter = await provider.connection.getTokenAccountBalance(
//         adminTokenAccount
//       );
//       console.log(
//         "Admin token balance after mint:",
//         balanceAfter.value.uiAmount
//       );

//       // Должно быть распределение: 10% админу и 90% в пул ликвидности
//       expect(balanceAfter.value.uiAmount).to.be.greaterThan(
//         balanceBefore.value.uiAmount
//       );

//       // Проверяем баланс пула ликвидности
//       const liquidityBalance = await provider.connection.getTokenAccountBalance(
//         liquidityPoolAccount
//       );
//       console.log("Liquidity pool balance:", liquidityBalance.value.uiAmount);
//       expect(liquidityBalance.value.uiAmount).to.be.greaterThan(0);
//     });

//     it("Mints directly to liquidity pool", async () => {
//       // Получаем баланс пула ликвидности до минта
//       const liquidityBalanceBefore =
//         await provider.connection.getTokenAccountBalance(liquidityPoolAccount);
//       console.log(
//         "Liquidity pool balance before mint:",
//         liquidityBalanceBefore.value.uiAmount
//       );

//       // Мокаем менеджера ликвидности
//       const liquidityManager = Keypair.generate().publicKey;

//       // Выполняем минтинг токенов напрямую в пул ликвидности
//       const mintAmount = 2_000_000 * 1_000_000_000; // 2 млн токенов с учетом decimals
//       const tx = await nDollarTokenProgram.methods
//         .mintToLiquidity(new anchor.BN(mintAmount))
//         .accounts({
//           authority: authority.publicKey,
//           mint: nDollarMint,
//           adminAccount: nDollarAdminAccount,
//           adminTokenAccount: adminTokenAccount,
//           liquidityPoolAccount: liquidityPoolAccount,
//           liquidityManager: liquidityManager,
//           tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
//           systemProgram: anchor.web3.SystemProgram.programId,
//         })
//         .rpc();

//       console.log("Mint to liquidity tx:", tx);

//       // Проверяем, что баланс пула ликвидности увеличился
//       const liquidityBalanceAfter =
//         await provider.connection.getTokenAccountBalance(liquidityPoolAccount);
//       console.log(
//         "Liquidity pool balance after mint:",
//         liquidityBalanceAfter.value.uiAmount
//       );
//       expect(liquidityBalanceAfter.value.uiAmount).to.be.greaterThan(
//         liquidityBalanceBefore.value.uiAmount
//       );
//     });

//     it("Adds and removes authorized signers", async () => {
//       // Создаем новый ключ для авторизованного подписанта
//       const newSigner = Keypair.generate().publicKey;

//       // Добавляем авторизованного подписанта
//       const addTx = await nDollarTokenProgram.methods
//         .addAuthorizedSigner(newSigner)
//         .accounts({
//           authority: authority.publicKey,
//           mint: nDollarMint,
//           adminAccount: nDollarAdminAccount,
//           tokenAccount: adminTokenAccount,
//           tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
//           systemProgram: anchor.web3.SystemProgram.programId,
//         })
//         .rpc();

//       console.log("Add authorized signer tx:", addTx);

//       // Проверяем, что подписант добавлен
//       let adminAccount = await nDollarTokenProgram.account.adminAccount.fetch(
//         nDollarAdminAccount
//       );
//       expect(adminAccount.authorizedSigners[0].toString()).to.equal(
//         newSigner.toString()
//       );

//       // Удаляем авторизованного подписанта
//       const removeTx = await nDollarTokenProgram.methods
//         .removeAuthorizedSigner(newSigner)
//         .accounts({
//           authority: authority.publicKey,
//           mint: nDollarMint,
//           adminAccount: nDollarAdminAccount,
//           tokenAccount: adminTokenAccount,
//           tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
//           systemProgram: anchor.web3.SystemProgram.programId,
//         })
//         .rpc();

//       console.log("Remove authorized signer tx:", removeTx);

//       // Проверяем, что подписант удален
//       adminAccount = await nDollarTokenProgram.account.adminAccount.fetch(
//         nDollarAdminAccount
//       );
//       expect(adminAccount.authorizedSigners[0]).to.be.null;
//     });

//     it("Sets minimum required signers", async () => {
//       // Устанавливаем минимальное количество подписантов
//       const minSigners = 2;
//       const tx = await nDollarTokenProgram.methods
//         .setMinRequiredSigners(minSigners)
//         .accounts({
//           authority: authority.publicKey,
//           mint: nDollarMint,
//           adminAccount: nDollarAdminAccount,
//           tokenAccount: adminTokenAccount,
//           tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
//           systemProgram: anchor.web3.SystemProgram.programId,
//         })
//         .rpc();

//       console.log("Set min required signers tx:", tx);

//       // Проверяем, что значение установлено правильно
//       const adminAccount = await nDollarTokenProgram.account.adminAccount.fetch(
//         nDollarAdminAccount
//       );
//       expect(adminAccount.minRequiredSigners).to.equal(minSigners);
//     });

//     it("Burns tokens", async () => {
//       // Получаем текущий supply перед сжиганием
//       const mintInfo = await getMint(provider.connection, nDollarMint);
//       const supplyBefore = mintInfo.supply;
//       console.log("Supply before burn:", supplyBefore.toString());

//       // Сжигаем токены
//       const burnAmount = 500_000 * 1_000_000_000; // 500k токенов с учетом decimals
//       const tx = await nDollarTokenProgram.methods
//         .burnTokens(new anchor.BN(burnAmount))
//         .accounts({
//           authority: authority.publicKey,
//           mint: nDollarMint,
//           adminAccount: nDollarAdminAccount,
//           tokenAccount: adminTokenAccount,
//           tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
//           systemProgram: anchor.web3.SystemProgram.programId,
//         })
//         .rpc();

//       console.log("Burn tokens tx:", tx);

//       // Проверяем, что supply уменьшился
//       const mintInfoAfter = await getMint(provider.connection, nDollarMint);
//       const supplyAfter = mintInfoAfter.supply;
//       console.log("Supply after burn:", supplyAfter.toString());
//       expect(Number(supplyAfter)).to.be.lessThan(Number(supplyBefore));
//     });
//   });
// });
