// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import { PublicKey, Keypair } from "@solana/web3.js";
// import { expect } from "chai";
// import {
//   createMint,
//   getMint,
//   createAssociatedTokenAccount,
//   getOrCreateAssociatedTokenAccount,
// } from "@solana/spl-token";

// describe("n-dollar-token", () => {
//   // Настройка провайдера Anchor
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//   // Программа N-Dollar Token
//   const nDollarTokenProgram = anchor.workspace.NDollarToken;

//   // Кошелек пользователя (админа)
//   const authority = provider.wallet;

//   // Mint для N-Dollar
//   let nDollarMint: PublicKey;

//   // PDA для admin_account N-Dollar
//   let nDollarAdminAccount: PublicKey;
//   let nDollarAdminAccountBump: number;

//   // Токен-аккаунт для админа
//   let adminTokenAccount: PublicKey;

//   // Токен-аккаунт для пула ликвидности
//   let liquidityPoolAccount: PublicKey;

//   // Мокаем менеджер ликвидности
//   const liquidityManager = Keypair.generate().publicKey;

//   before(async () => {
//     console.log("Setting up N-Dollar token tests...");

//     // Создаем mint для N-Dollar
//     nDollarMint = await createMint(
//       provider.connection,
//       provider.wallet.payer, // Payer
//       provider.wallet.publicKey, // Mint authority
//       provider.wallet.publicKey, // Freeze authority
//       9 // Decimals
//     );

//     console.log("N-Dollar Mint created:", nDollarMint.toString());

//     // Получаем PDA для admin_account
//     [nDollarAdminAccount, nDollarAdminAccountBump] =
//       PublicKey.findProgramAddressSync(
//         [Buffer.from("admin_account"), nDollarMint.toBytes()],
//         nDollarTokenProgram.programId
//       );

//     console.log("N-Dollar Admin Account PDA:", nDollarAdminAccount.toString());
//     console.log("N-Dollar Admin Account Bump:", nDollarAdminAccountBump);

//     // Создаем токен-аккаунт для администратора
//     const adminTokenAccountInfo = await getOrCreateAssociatedTokenAccount(
//       provider.connection,
//       provider.wallet.payer,
//       nDollarMint,
//       provider.wallet.publicKey
//     );
//     adminTokenAccount = adminTokenAccountInfo.address;

//     console.log("Admin Token Account:", adminTokenAccount.toString());

//     // Создаем токен-аккаунт для пула ликвидности (для тестирования используем отдельный аккаунт)
//     const liquidityKeypair = Keypair.generate();
//     const liquidityAccountInfo = await getOrCreateAssociatedTokenAccount(
//       provider.connection,
//       provider.wallet.payer,
//       nDollarMint,
//       liquidityKeypair.publicKey,
//       true // allowOwnerOffCurve = true
//     );
//     liquidityPoolAccount = liquidityAccountInfo.address;

//     console.log(
//       "Liquidity Pool Token Account:",
//       liquidityPoolAccount.toString()
//     );
//   });

//   it("Initializes N-Dollar token", async () => {
//     // Инициализируем токен N-Dollar
//     try {
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
//           adminConfig: null, // Опциональный параметр
//           adminControlProgram: null, // Опциональный параметр
//           systemProgram: anchor.web3.SystemProgram.programId,
//           tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
//           rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//         })
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
//     } catch (error) {
//       console.error("Error initializing N-Dollar token:", error);
//       throw error;
//     }
//   });

//   it("Mints supply according to schedule", async () => {
//     // Получаем баланс до минта
//     const balanceBefore = await provider.connection.getTokenAccountBalance(
//       adminTokenAccount
//     );
//     console.log(
//       "Admin token balance before mint:",
//       balanceBefore.value.uiAmount
//     );

//     // Выполняем минтинг токенов согласно расписанию
//     const mintAmount = 1_000_000 * 1_000_000_000; // 1 млн токенов с учетом decimals
//     try {
//       const tx = await nDollarTokenProgram.methods
//         .mintSupply(new anchor.BN(mintAmount))
//         .accounts({
//           authority: authority.publicKey,
//           mint: nDollarMint,
//           adminAccount: nDollarAdminAccount,
//           tokenAccount: adminTokenAccount,
//           liquidityPoolAccount: liquidityPoolAccount,
//           adminConfig: null,
//           adminControlProgram: null,
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
//     } catch (error) {
//       console.error("Error minting supply:", error);
//       throw error;
//     }
//   });

//   it("Mints directly to liquidity pool", async () => {
//     // Получаем баланс пула ликвидности до минта
//     const liquidityBalanceBefore =
//       await provider.connection.getTokenAccountBalance(liquidityPoolAccount);
//     console.log(
//       "Liquidity pool balance before mint:",
//       liquidityBalanceBefore.value.uiAmount
//     );

//     // Выполняем минтинг токенов напрямую в пул ликвидности
//     const mintAmount = 2_000_000 * 1_000_000_000; // 2 млн токенов с учетом decimals
//     try {
//       const tx = await nDollarTokenProgram.methods
//         .mintToLiquidity(new anchor.BN(mintAmount))
//         .accounts({
//           authority: authority.publicKey,
//           mint: nDollarMint,
//           adminAccount: nDollarAdminAccount,
//           adminTokenAccount: adminTokenAccount,
//           liquidityPoolAccount: liquidityPoolAccount,
//           liquidityManager: liquidityManager,
//           liquidityManagerProgram: null,
//           adminConfig: null,
//           adminControlProgram: null,
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
//     } catch (error) {
//       console.error("Error minting to liquidity:", error);
//       throw error;
//     }
//   });

//   it("Adds and removes authorized signers", async () => {
//     // Создаем новый ключ для авторизованного подписанта
//     const newSigner = Keypair.generate().publicKey;

//     try {
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
//     } catch (error) {
//       console.error("Error managing authorized signers:", error);
//       throw error;
//     }
//   });

//   it("Sets minimum required signers", async () => {
//     try {
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
//     } catch (error) {
//       console.error("Error setting min required signers:", error);
//       throw error;
//     }
//   });

//   it("Burns tokens", async () => {
//     try {
//       // Получаем текущий supply перед сжиганием
//       const mintInfo = await getMint(provider.connection, nDollarMint);
//       const supplyBefore = mintInfo.supply;
//       console.log("Supply before burn:", supplyBefore.toString());

//       // Если supply равен 0, сначала минтим токены
//       if (Number(supplyBefore) === 0) {
//         const mintAmount = 1_000_000 * 1_000_000_000; // 1 млн токенов
//         await nDollarTokenProgram.methods
//           .mintSupply(new anchor.BN(mintAmount))
//           .accounts({
//             authority: authority.publicKey,
//             mint: nDollarMint,
//             adminAccount: nDollarAdminAccount,
//             tokenAccount: adminTokenAccount,
//             liquidityPoolAccount: liquidityPoolAccount,
//             adminConfig: null,
//             adminControlProgram: null,
//             tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
//             systemProgram: anchor.web3.SystemProgram.programId,
//           })
//           .rpc();

//         console.log("Minted tokens before burn test");
//       }

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
//     } catch (error) {
//       console.error("Error burning tokens:", error);
//       throw error;
//     }
//   });
// });
