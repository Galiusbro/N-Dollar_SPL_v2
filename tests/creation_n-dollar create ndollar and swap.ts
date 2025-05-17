// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import { PublicKey, Keypair } from "@solana/web3.js";
// import {
//   TOKEN_PROGRAM_ID,
//   createAssociatedTokenAccountInstruction,
// } from "@solana/spl-token";
// import { assert } from "chai";
// import { BN } from "bn.js";

// describe("N-Dollar Token Creation Test", () => {
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//   const program = anchor.workspace.NDollar as Program;
//   const liquidityPoolProgram = anchor.workspace.LiquidityPool as Program;
//   const wallet = provider.wallet as anchor.Wallet;
//   const METADATA_PROGRAM_ID = new PublicKey(
//     "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
//   );

//   // Объявляем переменные без начальных значений
//   let mint: Keypair;
//   let metadataPda: PublicKey;

//   // Утилитная функция для проверки и создания ATA
//   async function getOrCreateATA(mint: PublicKey, owner: PublicKey, payer: any) {
//     const ata = await anchor.utils.token.associatedAddress({ mint, owner });

//     try {
//       await provider.connection.getTokenAccountBalance(ata);
//       console.log("ATA exists:", ata.toString());
//     } catch {
//       try {
//         const createATAIx = createAssociatedTokenAccountInstruction(
//           "payer" in payer ? payer.payer.publicKey : payer.publicKey,
//           ata,
//           owner,
//           mint
//         );
//         const tx = new anchor.web3.Transaction().add(createATAIx);
//         await provider.sendAndConfirm(tx);
//         console.log("Created new ATA:", ata.toString());
//       } catch (e) {
//         console.error("Error creating ATA:", e);
//       }
//     }
//     return ata;
//   }

//   // Добавляем функцию для аирдропа
//   async function airdropSol(address: PublicKey, amount: number) {
//     const signature = await provider.connection.requestAirdrop(address, amount);
//     await provider.connection.confirmTransaction(signature);
//     console.log(
//       `Airdropped ${
//         amount / anchor.web3.LAMPORTS_PER_SOL
//       } SOL to ${address.toString()}`
//     );
//   }

//   // Функция для чтения строки из буфера
//   function readMetaplexString(
//     buffer: Buffer,
//     offset: number
//   ): [string, number] {
//     // В формате Metaplex первые 4 байта - длина строки
//     const length = buffer.readUInt32LE(offset);
//     console.log(`Reading string at offset ${offset}, length: ${length}`);
//     if (length === 0) {
//       return ["", offset + 4];
//     }
//     const str = buffer.slice(offset + 4, offset + 4 + length).toString("utf8");
//     console.log(`Read string: "${str}"`);
//     return [str, offset + 4 + length];
//   }

//   before(async () => {
//     // Инициализируем все PDA и аккаунты перед тестами
//     mint = Keypair.generate();

//     [metadataPda] = PublicKey.findProgramAddressSync(
//       [
//         Buffer.from("metadata"),
//         METADATA_PROGRAM_ID.toBuffer(),
//         mint.publicKey.toBuffer(),
//       ],
//       METADATA_PROGRAM_ID
//     );

//     // Добавляем аирдроп для пользователя
//     const userBalance = await provider.connection.getBalance(wallet.publicKey);
//     if (userBalance < anchor.web3.LAMPORTS_PER_SOL) {
//       await airdropSol(wallet.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL); // 2 SOL для тестов
//     }
//     console.log(
//       "User SOL balance:",
//       (await provider.connection.getBalance(wallet.publicKey)) /
//         anchor.web3.LAMPORTS_PER_SOL,
//       "SOL"
//     );
//   });

//   it("Creates token with metadata", async () => {
//     const tx = await program.methods
//       .createToken("One-Click Token", "ONE", "https://oneclick.com/token.json")
//       .accounts({
//         mint: mint.publicKey,
//         metadata: metadataPda,
//         authority: wallet.publicKey,
//         tokenProgram: TOKEN_PROGRAM_ID,
//         systemProgram: anchor.web3.SystemProgram.programId,
//         rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//         tokenMetadataProgram: METADATA_PROGRAM_ID,
//       })
//       .signers([mint])
//       .rpc();

//     console.log("Create token tx:", tx);

//     // Проверяем, что минт создан
//     const mintInfo = await provider.connection.getAccountInfo(mint.publicKey);
//     console.log("Mint created:", mintInfo !== null);

//     // Проверяем, что метаданные созданы и выводим их
//     const metadataInfo = await provider.connection.getAccountInfo(metadataPda);
//     if (metadataInfo) {
//       try {
//         // Пропускаем первые 1 + 32 + 32 байта (версия + update authority + mint)
//         let offset = 1 + 32 + 32;

//         // Читаем имя
//         const [name, nameEnd] = readMetaplexString(metadataInfo.data, offset);

//         // Читаем символ
//         const [symbol, symbolEnd] = readMetaplexString(
//           metadataInfo.data,
//           nameEnd
//         );

//         // Читаем URI
//         const [uri] = readMetaplexString(metadataInfo.data, symbolEnd);

//         console.log("\nToken Metadata:");
//         console.log("Name:", name);
//         console.log("Symbol:", symbol);
//         console.log("URI:", uri);
//       } catch (error) {
//         console.log("Error reading metadata:", error);
//         console.log(
//           "Error details:",
//           error instanceof Error ? error.message : error
//         );
//       }
//     } else {
//       console.log("Metadata account not found");
//     }
//   });

//   it("Initializes liquidity pool", async () => {
//     // Находим PDA для пула
//     const [poolPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("pool"), mint.publicKey.toBuffer()],
//       new PublicKey("B24yupzEDjF7Z9frnDG16uAwH1ZYfB57kuzh8jwDsL83") // liquidity pool program ID
//     );

//     // Находим PDA для SOL vault
//     const [solVaultPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("sol_vault"), poolPda.toBuffer()],
//       new PublicKey("B24yupzEDjF7Z9frnDG16uAwH1ZYfB57kuzh8jwDsL83")
//     );

//     // Получаем адрес vault'а для N-Dollar токенов
//     const nDollarVault = await anchor.utils.token.associatedAddress({
//       mint: mint.publicKey,
//       owner: poolPda,
//     });

//     const tx = await program.methods
//       .initializeLiquidityPool()
//       .accounts({
//         mint: mint.publicKey,
//         pool: poolPda,
//         ndollarVault: nDollarVault,
//         solVault: solVaultPda,
//         authority: wallet.publicKey,
//         tokenProgram: TOKEN_PROGRAM_ID,
//         systemProgram: anchor.web3.SystemProgram.programId,
//         associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
//         rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//         liquidityPoolProgram: new PublicKey(
//           "B24yupzEDjF7Z9frnDG16uAwH1ZYfB57kuzh8jwDsL83"
//         ),
//       })
//       .rpc();

//     console.log("Initialize liquidity pool tx:", tx);

//     // Проверяем, что vault'ы созданы и имеют правильный баланс
//     const nDollarVaultAccount =
//       await provider.connection.getTokenAccountBalance(nDollarVault);
//     console.log("N-Dollar vault balance:", nDollarVaultAccount.value.uiAmount);

//     // Проверяем, что было заминчено правильное количество токенов (108,000,000)
//     const expectedAmount = 108_000_000;
//     assert.equal(
//       nDollarVaultAccount.value.uiAmount,
//       expectedAmount,
//       `Expected ${expectedAmount} tokens in vault, but got ${nDollarVaultAccount.value.uiAmount}`
//     );

//     const solVaultBalance = await provider.connection.getBalance(solVaultPda);
//     console.log(
//       "SOL vault balance:",
//       solVaultBalance / anchor.web3.LAMPORTS_PER_SOL,
//       "SOL"
//     );
//   });

//   it("Adds initial liquidity", async () => {
//     // Создаем или получаем ATA для пользователя
//     const userNDollarAccount = await getOrCreateATA(
//       mint.publicKey,
//       wallet.publicKey,
//       wallet
//     );

//     // Находим PDA для пула
//     const [poolPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("pool"), mint.publicKey.toBuffer()],
//       new PublicKey("B24yupzEDjF7Z9frnDG16uAwH1ZYfB57kuzh8jwDsL83")
//     );

//     // Находим PDA для SOL vault
//     const [solVaultPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("sol_vault"), poolPda.toBuffer()],
//       new PublicKey("B24yupzEDjF7Z9frnDG16uAwH1ZYfB57kuzh8jwDsL83")
//     );

//     // Получаем адрес vault'а для N-Dollar токенов пула
//     const nDollarVault = await anchor.utils.token.associatedAddress({
//       mint: mint.publicKey,
//       owner: poolPda,
//     });

//     // Добавляем начальную ликвидность SOL в пул
//     const solAmount = new BN(100 * anchor.web3.LAMPORTS_PER_SOL); // 100 SOL

//     const tx = await liquidityPoolProgram.methods
//       .addLiquidity(new BN(0), solAmount)
//       .accounts({
//         pool: poolPda,
//         ndollarMint: mint.publicKey,
//         ndollarVault: nDollarVault,
//         solVault: solVaultPda,
//         user: wallet.publicKey,
//         userNdollar: userNDollarAccount,
//         systemProgram: anchor.web3.SystemProgram.programId,
//         tokenProgram: TOKEN_PROGRAM_ID,
//       })
//       .rpc();

//     console.log("Add liquidity tx:", tx);

//     // Проверяем балансы после добавления ликвидности
//     const nDollarVaultBalance =
//       await provider.connection.getTokenAccountBalance(nDollarVault);
//     console.log("Pool N-Dollar balance:", nDollarVaultBalance.value.uiAmount);

//     const solVaultBalance = await provider.connection.getBalance(solVaultPda);
//     console.log(
//       "Pool SOL balance:",
//       solVaultBalance / anchor.web3.LAMPORTS_PER_SOL,
//       "SOL"
//     );
//   });

//   it("Swaps SOL to N-Dollar", async () => {
//     // Создаем или получаем ATA для пользователя
//     const userNDollarAccount = await getOrCreateATA(
//       mint.publicKey,
//       wallet.publicKey,
//       wallet
//     );

//     // Находим PDA для пула
//     const [poolPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("pool"), mint.publicKey.toBuffer()],
//       new PublicKey("B24yupzEDjF7Z9frnDG16uAwH1ZYfB57kuzh8jwDsL83")
//     );

//     // Находим PDA для SOL vault
//     const [solVaultPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("sol_vault"), poolPda.toBuffer()],
//       new PublicKey("B24yupzEDjF7Z9frnDG16uAwH1ZYfB57kuzh8jwDsL83")
//     );

//     // Получаем адрес vault'а для N-Dollar токенов пула
//     const nDollarVault = await anchor.utils.token.associatedAddress({
//       mint: mint.publicKey,
//       owner: poolPda,
//     });

//     const solAmount = new BN(1 * anchor.web3.LAMPORTS_PER_SOL); // 1 SOL

//     const tx = await liquidityPoolProgram.methods
//       .swapSolToNdollar(solAmount)
//       .accounts({
//         pool: poolPda,
//         ndollarMint: mint.publicKey,
//         ndollarVault: nDollarVault,
//         solVault: solVaultPda,
//         user: wallet.publicKey,
//         userNdollar: userNDollarAccount,
//         systemProgram: anchor.web3.SystemProgram.programId,
//         tokenProgram: TOKEN_PROGRAM_ID,
//       })
//       .rpc();

//     console.log("Swap SOL to N-Dollar tx:", tx);

//     // Проверяем балансы после свапа
//     const userNDollarBalance = await provider.connection.getTokenAccountBalance(
//       userNDollarAccount
//     );
//     console.log(
//       "User N-Dollar balance after swap:",
//       userNDollarBalance.value.uiAmount
//     );

//     const poolSolBalance = await provider.connection.getBalance(solVaultPda);
//     console.log(
//       "Pool SOL balance after swap:",
//       poolSolBalance / anchor.web3.LAMPORTS_PER_SOL,
//       "SOL"
//     );
//   });

//   it("Swaps N-Dollar to SOL", async () => {
//     // Создаем или получаем ATA для пользователя
//     const userNDollarAccount = await getOrCreateATA(
//       mint.publicKey,
//       wallet.publicKey,
//       wallet
//     );

//     // Находим PDA для пула
//     const [poolPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("pool"), mint.publicKey.toBuffer()],
//       new PublicKey("B24yupzEDjF7Z9frnDG16uAwH1ZYfB57kuzh8jwDsL83")
//     );

//     // Находим PDA для SOL vault
//     const [solVaultPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("sol_vault"), poolPda.toBuffer()],
//       new PublicKey("B24yupzEDjF7Z9frnDG16uAwH1ZYfB57kuzh8jwDsL83")
//     );

//     // Получаем адрес vault'а для N-Dollar токенов пула
//     const nDollarVault = await anchor.utils.token.associatedAddress({
//       mint: mint.publicKey,
//       owner: poolPda,
//     });

//     // Получаем текущий баланс N-Dollar пользователя
//     const userNDollarBalance = await provider.connection.getTokenAccountBalance(
//       userNDollarAccount
//     );
//     const ndollarAmount = new BN(userNDollarBalance.value.amount).divn(2); // Свапаем половину баланса

//     const userSolBalanceBefore = await provider.connection.getBalance(
//       wallet.publicKey
//     );

//     const tx = await liquidityPoolProgram.methods
//       .swapNdollarToSol(ndollarAmount)
//       .accounts({
//         pool: poolPda,
//         ndollarMint: mint.publicKey,
//         ndollarVault: nDollarVault,
//         solVault: solVaultPda,
//         user: wallet.publicKey,
//         userNdollar: userNDollarAccount,
//         systemProgram: anchor.web3.SystemProgram.programId,
//         tokenProgram: TOKEN_PROGRAM_ID,
//       })
//       .rpc();

//     console.log("Swap N-Dollar to SOL tx:", tx);

//     // Проверяем балансы после свапа
//     const userNDollarBalanceAfter =
//       await provider.connection.getTokenAccountBalance(userNDollarAccount);
//     console.log(
//       "User N-Dollar balance after swap:",
//       userNDollarBalanceAfter.value.uiAmount
//     );

//     const userSolBalanceAfter = await provider.connection.getBalance(
//       wallet.publicKey
//     );
//     console.log(
//       "User received SOL:",
//       (userSolBalanceAfter - userSolBalanceBefore) /
//         anchor.web3.LAMPORTS_PER_SOL,
//       "SOL"
//     );

//     const poolSolBalance = await provider.connection.getBalance(solVaultPda);
//     console.log(
//       "Pool SOL balance after swap:",
//       poolSolBalance / anchor.web3.LAMPORTS_PER_SOL,
//       "SOL"
//     );
//   });

//   it("Mints additional N-Dollar tokens", async () => {
//     // Находим PDA для пула
//     const [poolPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("pool"), mint.publicKey.toBuffer()],
//       liquidityPoolProgram.programId
//     );

//     // Получаем адрес vault'а для N-Dollar токенов пула
//     // Этот ATA должен был быть создан во время initializeLiquidityPool
//     const poolNDollarVault = await anchor.utils.token.associatedAddress({
//       mint: mint.publicKey,
//       owner: poolPda,
//     });

//     const initialBalancePoolVault =
//       await provider.connection.getTokenAccountBalance(poolNDollarVault);
//     const amountToMint = new BN(5000 * 10 ** 9); // Минтим 5000 токенов в пул

//     const tx = await program.methods
//       .mintAdditionalTokens(amountToMint)
//       .accounts({
//         mint: mint.publicKey,
//         mintAuthority: wallet.publicKey, // authority - это кошелек пользователя, который создал минт
//         recipientTokenAccount: poolNDollarVault, // Направляем токены в хранилище пула
//         tokenProgram: TOKEN_PROGRAM_ID,
//       })
//       .rpc();

//     console.log("Mint additional tokens to pool vault tx:", tx);

//     const finalBalancePoolVault =
//       await provider.connection.getTokenAccountBalance(poolNDollarVault);
//     console.log(
//       "Pool N-Dollar vault balance after minting:",
//       finalBalancePoolVault.value.uiAmount
//     );

//     const expectedBalancePoolVault = new BN(
//       initialBalancePoolVault.value.amount
//     ).add(amountToMint);
//     assert.equal(
//       finalBalancePoolVault.value.amount,
//       expectedBalancePoolVault.toString(),
//       `Expected ${new BN(expectedBalancePoolVault)
//         .div(new BN(10 ** 9))
//         .toString()} tokens in pool vault, but got ${
//         finalBalancePoolVault.value.uiAmount
//       }`
//     );
//   });

//   it("Burns user N-Dollar tokens", async () => {
//     // Создаем или получаем ATA для пользователя
//     const userNDollarAccount = await getOrCreateATA(
//       mint.publicKey,
//       wallet.publicKey,
//       wallet
//     );

//     const initialBalance = await provider.connection.getTokenAccountBalance(
//       userNDollarAccount
//     );
//     const amountToBurn = new BN(initialBalance.value.amount).divn(2); // Сжигаем половину баланса

//     if (new BN(initialBalance.value.amount).isZero()) {
//       console.log("User has no tokens to burn. Skipping burn test.");
//       return;
//     }
//     assert(
//       new BN(initialBalance.value.amount).gt(new BN(0)),
//       "User has no tokens to burn."
//     );

//     const tx = await program.methods
//       .burnUserTokens(amountToBurn)
//       .accounts({
//         mint: mint.publicKey,
//         userTokenAccount: userNDollarAccount,
//         owner: wallet.publicKey, // owner - это кошелек пользователя, который владеет токенами
//         tokenProgram: TOKEN_PROGRAM_ID,
//       })
//       .rpc();

//     console.log("Burn user tokens tx:", tx);

//     const finalBalance = await provider.connection.getTokenAccountBalance(
//       userNDollarAccount
//     );
//     console.log(
//       "User N-Dollar balance after burning:",
//       finalBalance.value.uiAmount
//     );

//     const expectedBalance = new BN(initialBalance.value.amount).sub(
//       amountToBurn
//     );
//     assert.equal(
//       finalBalance.value.amount,
//       expectedBalance.toString(),
//       `Expected ${expectedBalance.toNumber() / 10 ** 9} tokens, but got ${
//         finalBalance.value.uiAmount
//       }`
//     );
//   });
// });
