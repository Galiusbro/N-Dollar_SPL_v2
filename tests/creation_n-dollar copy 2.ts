// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import {
//   PublicKey,
//   Keypair,
//   SystemProgram,
//   SYSVAR_RENT_PUBKEY,
// } from "@solana/web3.js";
// import {
//   TOKEN_PROGRAM_ID,
//   ASSOCIATED_TOKEN_PROGRAM_ID,
//   getAssociatedTokenAddressSync,
//   getAccount,
//   createAssociatedTokenAccountInstruction, // Keep for initial setup if needed
// } from "@solana/spl-token";
// import { assert } from "chai";
// import { BN } from "bn.js";

// // Helper function to safely get token balance (returns 0 if account doesn't exist)
// async function getTokenBalance(
//   provider: anchor.Provider,
//   ata: PublicKey
// ): Promise<bigint> {
//   try {
//     const accountInfo = await getAccount(provider.connection, ata);
//     return accountInfo.amount;
//   } catch (error) {
//     // Check if the error is because the account doesn't exist
//     if (
//       error.message.includes("could not find account") ||
//       error.message.includes("Account does not exist")
//     ) {
//       console.log(`ATA ${ata.toString()} does not exist or has 0 balance.`);
//       return BigInt(0);
//     }
//     // Re-throw other errors
//     console.error(`Error fetching account ${ata.toString()}:`, error);
//     throw error;
//   }
// }

// const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

// describe("Token Creation and Distribution Test", () => {
//   // Program and account setup
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);
//   const wallet = provider.wallet as anchor.Wallet;

//   // --- ИЗМЕНЕНО: Загрузка всех необходимых программ ---
//   const tokenCreatorProgram = anchor.workspace.TokenCreator as Program;
//   const tokenDistributorProgram = anchor.workspace.TokenDistributor as Program; // Добавлено
//   const bondingCurveProgram = anchor.workspace.BondingCurve as Program; // Добавлено (даже если пустой, нужен ID)
//   const liquidityPoolProgram = anchor.workspace.LiquidityPool as Program;
//   // Оставляем NdollarProgram, если он все еще отвечает за создание N-Dollar и инициализацию пула
//   const NdollarProgram = anchor.workspace.NDollar as Program; // Убедитесь, что он еще есть и нужен

//   const METADATA_PROGRAM_ID = new PublicKey(
//     "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
//   );
//   const WRAPPED_SOL_MINT = new PublicKey(
//     "So11111111111111111111111111111111111111112"
//   );
//   const SOL_DECIMALS = anchor.web3.LAMPORTS_PER_SOL;

//   // N-Dollar Mint Keypair (оставляем как было)
//   const nDollarMintKp = Keypair.generate();
//   const nDollarMint = nDollarMintKp.publicKey;

//   // PDAs и ATAs для N-Dollar и пула (оставляем как было)
//   let nDollarMetadataPda: PublicKey;
//   let poolPda: PublicKey;
//   let solVaultPda: PublicKey;
//   let poolNDollarAccount: PublicKey; // ATA пула для N-Dollar
//   let userNDollarAccount: PublicKey; // ATA пользователя для N-Dollar

//   // Utility functions (оставляем)
//   // async function getOrCreateATA... (можно использовать getAssociatedTokenAddressSync + getAccount из spl-token)
//   async function airdropSol(address: PublicKey, amount: number) {
//     /* ... */
//   }
//   function readMetaplexString(
//     buffer: Buffer,
//     offset: number
//   ): [string, number] {
//     const length = buffer.readUInt32LE(offset);
//     if (length === 0) return ["", offset + 4];

//     const str = buffer.slice(offset + 4, offset + 4 + length).toString("utf8");
//     return [str, offset + 4 + length];
//   }

//   function parseMetadata(data: Buffer) {
//     try {
//       let offset = 1 + 32 + 32; // Skip past header
//       const [name, nameEnd] = readMetaplexString(data, offset);
//       const [symbol, symbolEnd] = readMetaplexString(data, nameEnd);
//       const [uri] = readMetaplexString(data, symbolEnd);

//       return { name, symbol, uri };
//     } catch (error) {
//       console.error("Error parsing metadata:", error);
//       return null;
//     }
//   }
//   function logTokenInfo(
//     name: string,
//     metadata: any,
//     balance: bigint | number | null = null
//   ) {
//     console.log(`\n${name} Token Info:`);
//     if (metadata) {
//       console.log("  Name:", metadata.name);
//       console.log("  Symbol:", metadata.symbol);
//       console.log("  URI:", metadata.uri);
//     }
//     if (balance !== null) {
//       // Форматируем bigint для читаемости
//       const balanceStr =
//         typeof balance === "bigint" ? balance.toString() : balance;
//       console.log("  Balance:", balanceStr);
//     }
//   }

//   // --- Добавляем переменные для User Token ---
//   let userTokenMintKp: Keypair;
//   let userTokenMint: PublicKey;
//   let userTokenMetadataPda: PublicKey;
//   let tokenInfoPda: PublicKey;
//   let distributorAuthorityPda: PublicKey;
//   let bondingCurveAuthorityPda: PublicKey;
//   let userTokenAccount: PublicKey; // ATA пользователя для User Token
//   let distributorTokenAccount: PublicKey; // ATA дистрибьютора для User Token
//   let bondingCurveTokenAccount: PublicKey; // ATA кривой для User Token
//   let bondingCurvePda: PublicKey; // PDA состояния кривой
//   let nDollarTreasury: PublicKey; // ATA казны кривой для N-Dollar
//   const tokenDecimals = 9; // Децималы мем-коина
//   const nDollarDecimals = 9; // Децималы N-Dollar (убедись, что они совпадают с созданием N-Dollar)
//   // // Initialize PDAs and accounts before tests
//   before(async () => {
//     // PDAs для N-Dollar
//     [nDollarMetadataPda] = PublicKey.findProgramAddressSync(
//       [
//         Buffer.from("metadata"),
//         METADATA_PROGRAM_ID.toBuffer(),
//         nDollarMint.toBuffer(),
//       ],
//       METADATA_PROGRAM_ID
//     );

//     // PDAs и ATAs для Пула Ликвидности
//     [poolPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("pool"), nDollarMint.toBuffer()],
//       liquidityPoolProgram.programId
//     );
//     [solVaultPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("sol_vault"), poolPda.toBuffer()],
//       liquidityPoolProgram.programId // Убедитесь, что poolPda используется как сид
//     );
//     poolNDollarAccount = getAssociatedTokenAddressSync(
//       nDollarMint,
//       poolPda,
//       true
//     ); // allowOwnerOffCurve = true для PDA
//     userNDollarAccount = getAssociatedTokenAddressSync(
//       nDollarMint,
//       wallet.publicKey
//     );

//     // Airdrop SOL
//     const userBalance = await provider.connection.getBalance(wallet.publicKey);
//     if (userBalance < 20 * SOL_DECIMALS) {
//       // Увеличил запас SOL на всякий случай
//       await airdropSol(wallet.publicKey, 20 * SOL_DECIMALS);
//     }
//     console.log(
//       "User SOL balance:",
//       (await provider.connection.getBalance(wallet.publicKey)) / SOL_DECIMALS,
//       "SOL"
//     );
//     console.log("N-Dollar Mint KP Pubkey:", nDollarMintKp.publicKey.toBase58());
//     console.log("Pool PDA:", poolPda.toBase58());
//     console.log("SOL Vault PDA:", solVaultPda.toBase58());
//     console.log("Pool N-Dollar ATA:", poolNDollarAccount.toBase58());
//     console.log("User N-Dollar ATA:", userNDollarAccount.toBase58());
//   });

//   // --- Шаги 1-5: Создание N-Dollar, инициализация и наполнение пула, свопы ---
//   // --- Оставляем эти шаги как были, если они работали ---

//   it("1. Creates N-Dollar token with metadata", async () => {
//     // Используем NdollarProgram, если он за это отвечает
//     const tx = await NdollarProgram.methods
//       .createToken("N-Dollar", "ND", "https://example.com/ndollar.json") // Пример данных
//       .accounts({
//         mint: nDollarMint,
//         metadata: nDollarMetadataPda,
//         authority: wallet.publicKey,
//         tokenProgram: TOKEN_PROGRAM_ID,
//         systemProgram: SystemProgram.programId,
//         rent: SYSVAR_RENT_PUBKEY,
//         tokenMetadataProgram: METADATA_PROGRAM_ID,
//       })
//       .signers([nDollarMintKp]) // Mint Keypair нужен как signer
//       .rpc({ commitment: "confirmed" }); // Добавил confirmed для надежности

//     console.log("Create N-Dollar token tx:", tx);
//     await provider.connection.confirmTransaction(tx, "confirmed");

//     const mintInfo = await provider.connection.getAccountInfo(nDollarMint);
//     assert(mintInfo !== null, "N-Dollar mint not created");
//     const metadataInfo = await provider.connection.getAccountInfo(
//       nDollarMetadataPda
//     );
//     assert(metadataInfo !== null, "Metadata account not found");
//     logTokenInfo("N-Dollar", parseMetadata(metadataInfo.data));
//   });

//   it("2. Initializes liquidity pool", async () => {
//     // Используем NdollarProgram или liquidityPoolProgram в зависимости от того, где эта логика
//     const tx = await NdollarProgram.methods // ИЛИ liquidityPoolProgram.methods.initialize(...)
//       .initializeLiquidityPool() // Убедитесь, что метод и аргументы верны
//       .accounts({
//         mint: nDollarMint, // N-Dollar mint
//         pool: poolPda,
//         ndollarVault: poolNDollarAccount, // ATA пула для N-Dollar
//         solVault: solVaultPda,
//         authority: wallet.publicKey, // Кто инициализирует
//         tokenProgram: TOKEN_PROGRAM_ID,
//         systemProgram: SystemProgram.programId,
//         associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//         rent: SYSVAR_RENT_PUBKEY,
//         liquidityPoolProgram: liquidityPoolProgram.programId, // ID программы пула
//       })
//       .rpc({ commitment: "confirmed" });
//     console.log("Initialize liquidity pool tx:", tx);
//     await provider.connection.confirmTransaction(tx, "confirmed");

//     // Проверка балансов после инициализации (если она что-то минтит)
//     const ndollarVaultBalance = await getTokenBalance(
//       provider,
//       poolNDollarAccount
//     );
//     console.log("Pool N-Dollar balance after init:", ndollarVaultBalance);
//     const solVaultBalance = await provider.connection.getBalance(solVaultPda);
//     console.log(
//       "Pool SOL balance after init:",
//       solVaultBalance / SOL_DECIMALS,
//       "SOL"
//     );

//     // Примерная проверка, если инициализация минтит 108M N-Dollar с 9 децималами
//     // const expectedAmount = BigInt("108000000000000000"); // 108M * 10^9
//     // assert.equal(ndollarVaultBalance, expectedAmount, `Expected ${expectedAmount} in pool vault`);
//   });

//   it("3. Creates user N-Dollar ATA (if needed) and adds initial liquidity", async () => {
//     await sleep(1000); // Добавляем задержку перед транзакцией

//     // Убедимся что ATA пользователя для N-Dollar существует
//     try {
//       await getAccount(provider.connection, userNDollarAccount);
//       console.log("User N-Dollar ATA already exists.");
//     } catch {
//       console.log("Creating User N-Dollar ATA...");
//       const createUserAtaTx = new anchor.web3.Transaction().add(
//         createAssociatedTokenAccountInstruction(
//           wallet.publicKey,
//           userNDollarAccount,
//           wallet.publicKey,
//           nDollarMint
//         )
//       );
//       const sig = await provider.sendAndConfirm(createUserAtaTx);
//       console.log("Create User N-Dollar ATA tx:", sig);
//     }

//     await sleep(1000); // Добавляем задержку перед следующей транзакцией

//     // Добавляем ликвидность (например, 10 SOL)
//     const solAmountToAdd = new BN(10 * SOL_DECIMALS);
//     const nDollarAmountToAdd = new BN(0);

//     const tx = await liquidityPoolProgram.methods
//       .addLiquidity(nDollarAmountToAdd, solAmountToAdd)
//       .accounts({
//         pool: poolPda,
//         ndollarMint: nDollarMint,
//         ndollarVault: poolNDollarAccount,
//         solVault: solVaultPda,
//         user: wallet.publicKey,
//         userNdollar: userNDollarAccount,
//         systemProgram: SystemProgram.programId,
//         tokenProgram: TOKEN_PROGRAM_ID,
//       })
//       .rpc({ commitment: "confirmed" });
//     console.log("Add liquidity tx:", tx);
//     await provider.connection.confirmTransaction(tx, "confirmed");

//     const poolNDollarBalance = await getTokenBalance(
//       provider,
//       poolNDollarAccount
//     );
//     console.log(
//       "Pool N-Dollar balance after add liquidity:",
//       poolNDollarBalance
//     );
//     const poolSolBalance = await provider.connection.getBalance(solVaultPda);
//     console.log(
//       "Pool SOL balance after add liquidity:",
//       poolSolBalance / SOL_DECIMALS,
//       "SOL"
//     );
//   });

//   it("4. Swaps SOL to N-Dollar", async () => {
//     await sleep(1000);

//     // Проверяем балансы
//     const poolSolBalance = await provider.connection.getBalance(solVaultPda);
//     const poolNDollarBalance = await getTokenBalance(
//       provider,
//       poolNDollarAccount
//     );
//     console.log(
//       "Pool SOL balance before swap:",
//       poolSolBalance / SOL_DECIMALS,
//       "SOL"
//     );
//     console.log(
//       "Pool N-Dollar balance before swap:",
//       poolNDollarBalance.toString()
//     );

//     const solToSwap = new BN(1 * SOL_DECIMALS);
//     const userNDollarBalanceBefore = await getTokenBalance(
//       provider,
//       userNDollarAccount
//     );
//     console.log(
//       "User N-Dollar balance before swap:",
//       userNDollarBalanceBefore.toString()
//     );

//     const tx = await liquidityPoolProgram.methods
//       .swapSolToNdollar(solToSwap)
//       .accounts({
//         pool: poolPda,
//         ndollarMint: nDollarMint,
//         ndollarVault: poolNDollarAccount,
//         solVault: solVaultPda,
//         user: wallet.publicKey,
//         userNdollar: userNDollarAccount,
//         systemProgram: SystemProgram.programId,
//         tokenProgram: TOKEN_PROGRAM_ID,
//       })
//       .rpc({ commitment: "confirmed" });
//     console.log("Swap SOL to N-Dollar tx:", tx);
//     await provider.connection.confirmTransaction(tx, "confirmed");

//     const userNDollarBalanceAfter = await getTokenBalance(
//       provider,
//       userNDollarAccount
//     );
//     const ndollarReceived = userNDollarBalanceAfter - userNDollarBalanceBefore;
//     console.log(
//       "User N-Dollar balance after swap:",
//       userNDollarBalanceAfter.toString()
//     );
//     console.log("User received N-Dollar:", ndollarReceived.toString());
//     assert(ndollarReceived > 0, "Should have received some N-Dollars");

//     const finalPoolSolBalance = await provider.connection.getBalance(
//       solVaultPda
//     );
//     console.log(
//       "Pool SOL balance after swap:",
//       finalPoolSolBalance / SOL_DECIMALS,
//       "SOL"
//     );
//   });

//   // Шаг 5 (Swap N-Dollar to SOL) оставляем как есть для полноты картины,
//   // хотя он не строго обязателен для теста createUserToken,
//   // но полезен для проверки работы пула в обе стороны.
//   // it("5. Swaps N-Dollar to SOL", async () => { ... });

//   // --- Шаг 6: Создание пользовательского токена с использованием N-Dollar ---
//   it("6. Creates user token using N-Dollars and distributes", async () => {
//     await sleep(1000);

//     const userTokenMintKp = Keypair.generate();
//     const userTokenMint = userTokenMintKp.publicKey;
//     console.log("\nCreating user token...");
//     console.log("User token mint pubkey:", userTokenMint.toString());

//     // Определяем необходимую сумму N-Dollar для аренды
//     const nDollarAmountForRent = new BN(50_000_000); // 50 миллионов ламелей N-Dollar = 0.05 N-Dollar

//     // Проверяем баланс N-Dollar перед созданием токена
//     const userNDollarBalanceBefore = await getTokenBalance(
//       provider,
//       userNDollarAccount
//     );
//     console.log(
//       `User N-Dollar balance before creation: ${userNDollarBalanceBefore.toString()}`
//     );

//     // Если баланс недостаточен, пытаемся получить больше N-Dollar
//     if (userNDollarBalanceBefore < nDollarAmountForRent.toNumber()) {
//       console.log("Insufficient N-Dollar balance, attempting to get more...");
//       const solToSwap = new BN(2 * SOL_DECIMALS); // Увеличиваем количество SOL для свапа

//       const swapTx = await liquidityPoolProgram.methods
//         .swapSolToNdollar(solToSwap)
//         .accounts({
//           pool: poolPda,
//           ndollarMint: nDollarMint,
//           ndollarVault: poolNDollarAccount,
//           solVault: solVaultPda,
//           user: wallet.publicKey,
//           userNdollar: userNDollarAccount,
//           systemProgram: SystemProgram.programId,
//           tokenProgram: TOKEN_PROGRAM_ID,
//         })
//         .rpc({ commitment: "confirmed" });

//       console.log("Additional swap tx:", swapTx);
//       await provider.connection.confirmTransaction(swapTx, "confirmed");

//       // Проверяем новый баланс
//       const newBalance = await getTokenBalance(provider, userNDollarAccount);
//       console.log(`New N-Dollar balance after swap: ${newBalance.toString()}`);

//       if (newBalance < nDollarAmountForRent.toNumber()) {
//         throw new Error(
//           `Still insufficient N-Dollar balance. Need ${nDollarAmountForRent.toString()}, have ${newBalance.toString()}`
//         );
//       }
//     }

//     assert(
//       userNDollarBalanceBefore >= nDollarAmountForRent.toNumber(),
//       `Insufficient N-Dollar balance. Need ${nDollarAmountForRent.toString()}, have ${userNDollarBalanceBefore.toString()}`
//     );

//     // --- ИЗМЕНЕНО: Находим ВСЕ необходимые PDA и ATA для НОВОГО токена ---

//     // PDA для метаданных (как раньше)
//     const [userTokenMetadataPda] = PublicKey.findProgramAddressSync(
//       [
//         Buffer.from("metadata"),
//         METADATA_PROGRAM_ID.toBuffer(),
//         userTokenMint.toBuffer(),
//       ],
//       METADATA_PROGRAM_ID
//     );

//     // PDA для информации о токене (как раньше, но с ID новой программы tokenCreator)
//     const [tokenInfoPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("token_info"), userTokenMint.toBuffer()],
//       tokenCreatorProgram.programId // Используем ID программы token_creator
//     );

//     // PDA для дистрибьютора
//     const [distributorAuthorityPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("distributor"), userTokenMint.toBuffer()],
//       tokenDistributorProgram.programId // ID программы token_distributor
//     );

//     // PDA для бондинг кривой
//     const [bondingCurveAuthorityPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("bonding_curve"), userTokenMint.toBuffer()],
//       bondingCurveProgram.programId // ID программы bonding_curve
//     );

//     // ATA для нового токена
//     const userTokenAccount = getAssociatedTokenAddressSync(
//       userTokenMint,
//       wallet.publicKey
//     );
//     const distributorTokenAccount = getAssociatedTokenAddressSync(
//       userTokenMint,
//       distributorAuthorityPda,
//       true
//     );
//     const bondingCurveTokenAccount = getAssociatedTokenAddressSync(
//       userTokenMint,
//       bondingCurveAuthorityPda,
//       true
//     );

//     console.log("User Token Mint:", userTokenMint.toBase58());
//     console.log("User Token Metadata PDA:", userTokenMetadataPda.toBase58());
//     console.log("Token Info PDA:", tokenInfoPda.toBase58());
//     console.log(
//       "Distributor Authority PDA:",
//       distributorAuthorityPda.toBase58()
//     );
//     console.log(
//       "Bonding Curve Authority PDA:",
//       bondingCurveAuthorityPda.toBase58()
//     );
//     console.log("User Token ATA:", userTokenAccount.toBase58());
//     console.log("Distributor Token ATA:", distributorTokenAccount.toBase58());
//     console.log(
//       "Bonding Curve Token ATA:",
//       bondingCurveTokenAccount.toBase58()
//     );

//     // --- ИЗМЕНЕНО: Параметры для createUserToken ---
//     const name = "My Distributed Token";
//     const symbol = "DIST";
//     const uri = "https://example.com/dist-token.json";
//     const totalSupply = new BN(1_000_000_000).mul(new BN(10 ** 9)); // 1 миллиард токенов с 9 децималами
//     // ВАЖНО: Сумма N-Dollar должна быть достаточной для аренды ВСЕХ аккаунтов
//     // mint, metadata, token_info, user ATA, distributor ATA, bonding curve ATA
//     // Используем большее значение для надежности теста, например, 0.05 N-Dollar (если децималы 9)

//     try {
//       const txSignature = await tokenCreatorProgram.methods
//         .createUserToken(name, symbol, uri, totalSupply, nDollarAmountForRent)
//         .accounts({
//           // Аккаунты создания токена
//           mint: userTokenMint,
//           metadata: userTokenMetadataPda,
//           tokenInfo: tokenInfoPda,
//           authority: wallet.publicKey, // Payer и user authority

//           // Аккаунты пула ликвидности (для свопа N-Dollar -> SOL)
//           liquidityPool: poolPda,
//           poolNDollarAccount: poolNDollarAccount, // Vault пула для N-Dollar
//           poolSolAccount: solVaultPda, // Vault пула для SOL
//           userNDollarAccount: userNDollarAccount, // ATA пользователя для N-Dollar
//           nDollarMint: nDollarMint, // Mint N-Dollar
//           solMint: WRAPPED_SOL_MINT, // Используется как 'placeholder' в Rust, но должен быть реальный mint (хоть и не используется напрямую в логике без WSOL)

//           // --- ИЗМЕНЕНО: Добавлены аккаунты для дистрибуции ---
//           distributorAuthority: distributorAuthorityPda,
//           distributorTokenAccount: distributorTokenAccount,
//           userTokenAccount: userTokenAccount, // ATA пользователя для *нового* токена
//           bondingCurveAuthority: bondingCurveAuthorityPda,
//           bondingCurveTokenAccount: bondingCurveTokenAccount,

//           // Системные программы и Rent
//           tokenProgram: TOKEN_PROGRAM_ID,
//           associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//           systemProgram: SystemProgram.programId,
//           rent: SYSVAR_RENT_PUBKEY,
//           tokenMetadataProgram: METADATA_PROGRAM_ID,

//           // --- ИЗМЕНЕНО: Добавлены ID программ для CPI и проверки PDA ---
//           liquidityPoolProgram: liquidityPoolProgram.programId,
//           tokenDistributorProgram: tokenDistributorProgram.programId,
//           bondingCurveProgram: bondingCurveProgram.programId, // Нужен для проверки сидов bondingCurveAuthority
//         })
//         .preInstructions([
//           anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
//             units: 400_000,
//           }),
//         ])
//         .signers([userTokenMintKp])
//         .rpc({ commitment: "confirmed" });

//       console.log("Create user token tx:", txSignature);

//       // --- ИЗМЕНЕНО: Проверка результатов дистрибуции ---
//       const userTokenMintInfo = await provider.connection.getAccountInfo(
//         userTokenMint
//       );
//       assert(userTokenMintInfo, "User Token mint should exist");

//       const metadataInfo = await provider.connection.getAccountInfo(
//         userTokenMetadataPda
//       );
//       assert(metadataInfo, "User Token metadata should exist");
//       const parsedMeta = parseMetadata(metadataInfo.data);

//       const userAtaBalance = await getTokenBalance(provider, userTokenAccount);
//       const bondingCurveAtaBalance = await getTokenBalance(
//         provider,
//         bondingCurveTokenAccount
//       );
//       const distributorAtaBalance = await getTokenBalance(
//         provider,
//         distributorTokenAccount
//       );

//       const expectedUserAmount = BigInt(
//         totalSupply.muln(70).divn(100).toString()
//       );
//       const expectedBondingCurveAmount = BigInt(
//         totalSupply.muln(30).divn(100).toString()
//       );

//       // Проверяем баланс N-Dollar после создания токена
//       const userNDollarBalanceAfter = await getTokenBalance(
//         provider,
//         userNDollarAccount
//       );
//       const spentNDollars = userNDollarBalanceBefore - userNDollarBalanceAfter;

//       // Логгирование для отладки
//       console.log("\nToken Creation Results:");
//       console.log("----------------------");
//       logTokenInfo("User Token", parsedMeta, userAtaBalance);

//       console.log("\nToken Distribution:");
//       console.log("-------------------");
//       console.log("Total Supply:", totalSupply.toString());
//       console.log(
//         "Bonding Curve Balance (30%):",
//         bondingCurveAtaBalance.toString()
//       );
//       console.log("Distributor Balance:", distributorAtaBalance.toString());
//       console.log("\nN-Dollar Usage:");
//       console.log("---------------");
//       console.log(
//         "Initial N-Dollar Balance:",
//         userNDollarBalanceBefore.toString()
//       );
//       console.log(
//         "Final N-Dollar Balance:",
//         userNDollarBalanceAfter.toString()
//       );
//       console.log("Spent N-Dollars:", spentNDollars.toString());

//       // Проверки
//       assert.equal(
//         userAtaBalance,
//         expectedUserAmount,
//         "User ATA balance mismatch (70%)"
//       );
//       assert.equal(
//         bondingCurveAtaBalance,
//         expectedBondingCurveAmount,
//         "Bonding Curve ATA balance mismatch (30%)"
//       );
//       assert.equal(
//         distributorAtaBalance,
//         BigInt(0),
//         "Distributor ATA should be empty"
//       );

//       // Проверяем, что потрачено примерно nDollarAmountForRent (с допуском 10%)
//       const maxAllowedSpent =
//         (BigInt(nDollarAmountForRent.toString()) * BigInt(110)) / BigInt(100);
//       assert(
//         spentNDollars > 0 && spentNDollars <= maxAllowedSpent,
//         `N-Dollar spent amount (${spentNDollars}) is outside allowed range (0, ${maxAllowedSpent})`
//       );
//     } catch (error) {
//       console.error("Error creating user token:", error);
//       if (error.logs) {
//         console.error("Transaction Logs:", error.logs);
//       }
//       throw error;
//     }
//   });

//   // --- Шаг 7: Инициализация Бондинг Кривой ---
//   it("7. Initializes the bonding curve", async () => {
//     await sleep(1000);
//     console.log("\nInitializing bonding curve for:", userTokenMint.toBase58());

//     // Проверяем, что нужные переменные userTokenMint, nDollarMint, etc. доступны из предыдущего шага
//     assert(userTokenMint, "userTokenMint not set");
//     assert(bondingCurvePda, "bondingCurvePda not set");
//     assert(bondingCurveTokenAccount, "bondingCurveTokenAccount not set");
//     assert(nDollarTreasury, "nDollarTreasury not set");
//     assert(bondingCurveAuthorityPda, "bondingCurveAuthorityPda not set");

//     try {
//       const txSignature = await bondingCurveProgram.methods
//         .initializeCurve()
//         .accounts({
//           bondingCurve: bondingCurvePda, // PDA состояния (создается)
//           mint: userTokenMint,
//           nDollarMint: nDollarMint,
//           bondingCurveTokenAccount: bondingCurveTokenAccount, // ATA с 30% токенов (уже существует)
//           nDollarTreasury: nDollarTreasury, // ATA казны (создается)
//           bondingCurveAuthority: bondingCurveAuthorityPda, // PDA авторитета (для казны)
//           authority: wallet.publicKey, // Payer
//           systemProgram: SystemProgram.programId,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           rent: SYSVAR_RENT_PUBKEY,
//         })
//         .rpc({ commitment: "confirmed" });

//       console.log("Initialize bonding curve tx:", txSignature);
//       await provider.connection.confirmTransaction(txSignature, "confirmed");

//       // Проверяем, что аккаунт кривой создан
//       const curveAccount = await bondingCurveProgram.account.bondingCurve.fetch(
//         bondingCurvePda
//       );
//       assert(curveAccount, "Bonding curve state account not found");
//       assert.equal(
//         curveAccount.isInitialized,
//         true,
//         "Bonding curve not initialized"
//       );
//       assert.equal(
//         curveAccount.mint.toBase58(),
//         userTokenMint.toBase58(),
//         "Mint mismatch"
//       );
//       assert.equal(
//         curveAccount.nDollarMint.toBase58(),
//         nDollarMint.toBase58(),
//         "N-Dollar mint mismatch"
//       );
//       console.log("Bonding curve state initialized:");
//       console.log("  Slope Numerator:", curveAccount.slopeNumerator.toString());
//       console.log(
//         "  Slope Denominator:",
//         curveAccount.slopeDenominator.toString()
//       );
//       console.log(
//         "  Intercept Scaled:",
//         curveAccount.interceptScaled.toString()
//       );
//       console.log(
//         "  Initial Supply:",
//         curveAccount.initialBondingSupply.toString()
//       );

//       // Проверяем, что казна создана и пуста
//       const treasuryBalance = await getTokenBalance(provider, nDollarTreasury);
//       assert.equal(
//         treasuryBalance,
//         BigInt(0),
//         "N-Dollar treasury should be empty initially"
//       );
//       console.log("N-Dollar treasury created and empty.");
//     } catch (error) {
//       console.error("Error initializing bonding curve:", error);
//       if (error.logs) {
//         console.error("Logs:", error.logs);
//       }
//       throw error;
//     }
//   });

//   // --- Шаг 8: Покупка Токенов с Кривой ---
//   it("8. Buys tokens from the bonding curve", async () => {
//     await sleep(1000);
//     console.log("\nBuying tokens from bonding curve...");

//     // Сколько токенов покупаем (например, 1000 с децималами)
//     const amountToBuy = new BN(1000).mul(new BN(10 ** tokenDecimals));
//     const amountToBuyBigInt = BigInt(amountToBuy.toString());

//     // Получаем балансы ДО покупки
//     const userTokenBalanceBefore = await getTokenBalance(
//       provider,
//       userTokenAccount
//     );
//     const curveTokenBalanceBefore = await getTokenBalance(
//       provider,
//       bondingCurveTokenAccount
//     );
//     const userNDollarBalanceBefore = await getTokenBalance(
//       provider,
//       userNDollarAccount
//     );
//     const treasuryBalanceBefore = await getTokenBalance(
//       provider,
//       nDollarTreasury
//     );

//     console.log(`Attempting to buy ${amountToBuy.toString()} tokens`);
//     console.log("Balances BEFORE buy:");
//     console.log("  User Tokens:", userTokenBalanceBefore.toString());
//     console.log("  Curve Tokens:", curveTokenBalanceBefore.toString());
//     console.log("  User N-Dollars:", userNDollarBalanceBefore.toString());
//     console.log("  Treasury N-Dollars:", treasuryBalanceBefore.toString());

//     assert(userNDollarBalanceBefore > 0, "User has no N-Dollars to buy"); // Простая проверка

//     try {
//       const txSignature = await bondingCurveProgram.methods
//         .buy(amountToBuy) // Передаем BN
//         .accounts({
//           bondingCurve: bondingCurvePda,
//           mint: userTokenMint,
//           nDollarMint: nDollarMint,
//           bondingCurveTokenAccount: bondingCurveTokenAccount,
//           nDollarTreasury: nDollarTreasury,
//           userTokenAccount: userTokenAccount,
//           userNDollarAccount: userNDollarAccount,
//           bondingCurveAuthority: bondingCurveAuthorityPda, // PDA
//           userAuthority: wallet.publicKey, // Signer
//           tokenProgram: TOKEN_PROGRAM_ID,
//         })
//         .rpc({ commitment: "confirmed" });

//       console.log("Buy transaction tx:", txSignature);
//       await provider.connection.confirmTransaction(txSignature, "confirmed");

//       // Получаем балансы ПОСЛЕ покупки
//       const userTokenBalanceAfter = await getTokenBalance(
//         provider,
//         userTokenAccount
//       );
//       const curveTokenBalanceAfter = await getTokenBalance(
//         provider,
//         bondingCurveTokenAccount
//       );
//       const userNDollarBalanceAfter = await getTokenBalance(
//         provider,
//         userNDollarAccount
//       );
//       const treasuryBalanceAfter = await getTokenBalance(
//         provider,
//         nDollarTreasury
//       );

//       console.log("\nBalances AFTER buy:");
//       console.log("  User Tokens:", userTokenBalanceAfter.toString());
//       console.log("  Curve Tokens:", curveTokenBalanceAfter.toString());
//       console.log("  User N-Dollars:", userNDollarBalanceAfter.toString());
//       console.log("  Treasury N-Dollars:", treasuryBalanceAfter.toString());

//       const tokensReceived = userTokenBalanceAfter - userTokenBalanceBefore;
//       const tokensSent = curveTokenBalanceBefore - curveTokenBalanceAfter;
//       const nDollarsSpent = userNDollarBalanceBefore - userNDollarBalanceAfter;
//       const nDollarsReceivedTreasury =
//         treasuryBalanceAfter - treasuryBalanceBefore;

//       // Проверки
//       assert.equal(
//         tokensReceived,
//         amountToBuyBigInt,
//         "User did not receive correct amount of tokens"
//       );
//       assert.equal(
//         tokensSent,
//         amountToBuyBigInt,
//         "Curve did not send correct amount of tokens"
//       );
//       assert(nDollarsSpent > 0, "User did not spend any N-Dollars");
//       assert.equal(
//         nDollarsReceivedTreasury,
//         nDollarsSpent,
//         "Treasury did not receive the spent N-Dollars"
//       );

//       console.log(`User received ${tokensReceived} tokens.`);
//       console.log(`User spent ${nDollarsSpent} N-Dollar lamports.`);
//     } catch (error) {
//       console.error("Error buying from bonding curve:", error);
//       if (error.logs) {
//         console.error("Logs:", error.logs);
//       }
//       throw error;
//     }
//   });

//   // --- Шаг 9: Продажа Токенов на Кривую ---
//   it("9. Sells tokens to the bonding curve", async () => {
//     await sleep(1000);
//     console.log("\nSelling tokens to bonding curve...");

//     // Сколько токенов продаем (например, половину купленных)
//     const userTokenBalanceBeforeSell = await getTokenBalance(
//       provider,
//       userTokenAccount
//     );
//     const amountToSell = new BN(userTokenBalanceBeforeSell.toString()).divn(2); // Продаем половину
//     const amountToSellBigInt = BigInt(amountToSell.toString());

//     assert(amountToSellBigInt > 0, "Cannot sell zero tokens");

//     // Получаем балансы ДО продажи
//     const curveTokenBalanceBefore = await getTokenBalance(
//       provider,
//       bondingCurveTokenAccount
//     );
//     const userNDollarBalanceBefore = await getTokenBalance(
//       provider,
//       userNDollarAccount
//     );
//     const treasuryBalanceBefore = await getTokenBalance(
//       provider,
//       nDollarTreasury
//     );

//     console.log(`Attempting to sell ${amountToSell.toString()} tokens`);
//     console.log("Balances BEFORE sell:");
//     console.log("  User Tokens:", userTokenBalanceBeforeSell.toString());
//     console.log("  Curve Tokens:", curveTokenBalanceBefore.toString());
//     console.log("  User N-Dollars:", userNDollarBalanceBefore.toString());
//     console.log("  Treasury N-Dollars:", treasuryBalanceBefore.toString());

//     assert(treasuryBalanceBefore > 0, "Treasury has no N-Dollars to pay"); // Проверка казны

//     try {
//       const txSignature = await bondingCurveProgram.methods
//         .sell(amountToSell) // Передаем BN
//         .accounts({
//           bondingCurve: bondingCurvePda,
//           mint: userTokenMint,
//           nDollarMint: nDollarMint,
//           bondingCurveTokenAccount: bondingCurveTokenAccount,
//           nDollarTreasury: nDollarTreasury,
//           userTokenAccount: userTokenAccount,
//           userNDollarAccount: userNDollarAccount,
//           bondingCurveAuthority: bondingCurveAuthorityPda, // PDA
//           userAuthority: wallet.publicKey, // Signer
//           tokenProgram: TOKEN_PROGRAM_ID,
//         })
//         .rpc({ commitment: "confirmed" });

//       console.log("Sell transaction tx:", txSignature);
//       await provider.connection.confirmTransaction(txSignature, "confirmed");

//       // Получаем балансы ПОСЛЕ продажи
//       const userTokenBalanceAfter = await getTokenBalance(
//         provider,
//         userTokenAccount
//       );
//       const curveTokenBalanceAfter = await getTokenBalance(
//         provider,
//         bondingCurveTokenAccount
//       );
//       const userNDollarBalanceAfter = await getTokenBalance(
//         provider,
//         userNDollarAccount
//       );
//       const treasuryBalanceAfter = await getTokenBalance(
//         provider,
//         nDollarTreasury
//       );

//       console.log("\nBalances AFTER sell:");
//       console.log("  User Tokens:", userTokenBalanceAfter.toString());
//       console.log("  Curve Tokens:", curveTokenBalanceAfter.toString());
//       console.log("  User N-Dollars:", userNDollarBalanceAfter.toString());
//       console.log("  Treasury N-Dollars:", treasuryBalanceAfter.toString());

//       const tokensSold = userTokenBalanceBeforeSell - userTokenBalanceAfter;
//       const tokensReceivedCurve =
//         curveTokenBalanceAfter - curveTokenBalanceBefore;
//       const nDollarsReceived =
//         userNDollarBalanceAfter - userNDollarBalanceBefore;
//       const nDollarsSpentTreasury =
//         treasuryBalanceBefore - treasuryBalanceAfter;

//       // Проверки
//       assert.equal(
//         tokensSold,
//         amountToSellBigInt,
//         "User did not sell correct amount of tokens"
//       );
//       assert.equal(
//         tokensReceivedCurve,
//         amountToSellBigInt,
//         "Curve did not receive correct amount of tokens"
//       );
//       assert(nDollarsReceived > 0, "User did not receive any N-Dollars");
//       assert.equal(
//         nDollarsSpentTreasury,
//         nDollarsReceived,
//         "Treasury did not spend the received N-Dollars"
//       );

//       console.log(`User sold ${tokensSold} tokens.`);
//       console.log(`User received ${nDollarsReceived} N-Dollar lamports.`);
//     } catch (error) {
//       console.error("Error selling to bonding curve:", error);
//       if (error.logs) {
//         console.error("Logs:", error.logs);
//       }
//       throw error;
//     }
//   });
// });
