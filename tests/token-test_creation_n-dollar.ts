// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import { PublicKey, Keypair } from "@solana/web3.js";
// import {
//   TOKEN_PROGRAM_ID,
//   getAssociatedTokenAddress,
//   ASSOCIATED_TOKEN_PROGRAM_ID,
//   createAssociatedTokenAccountInstruction,
// } from "@solana/spl-token";
// import { assert } from "chai";
// import BN from "bn.js";
// import { Transaction } from "@solana/web3.js";

// describe("Simplified Token and Liquidity Pool Test", () => {
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//   const program = anchor.workspace.NDollar as Program;
//   const liquidityPoolProgram = anchor.workspace.LiquidityPool as Program;
//   const wallet = provider.wallet as anchor.Wallet;
//   const TOKEN_DECIMALS = 9;

//   // Объявляем переменные без начальных значений
//   let mint: Keypair;
//   let metadataPda: PublicKey;
//   let poolTokenAccount: PublicKey;
//   let solAccountPda: PublicKey;
//   let userTokenAccount: PublicKey;
//   let poolAuthorityPda: PublicKey;
//   let mintSchedulePda: PublicKey;

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

//   // Добавим функцию для вывода состояния пула
//   async function logPoolState(message: string) {
//     const poolTokenBalance = await provider.connection.getTokenAccountBalance(
//       poolTokenAccount
//     );
//     const solBalance = await provider.connection.getBalance(solAccountPda);
//     const price = await liquidityPoolProgram.methods
//       .getPrice()
//       .accounts({
//         poolAccount: poolTokenAccount,
//         solAccount: solAccountPda,
//       })
//       .view();

//     console.log("\n=== " + message + " ===");
//     console.log(
//       "Pool tokens:",
//       Number(poolTokenBalance.value.uiAmountString).toLocaleString()
//     );
//     console.log(
//       "Pool SOL:",
//       (solBalance / anchor.web3.LAMPORTS_PER_SOL).toLocaleString(),
//       "SOL"
//     );
//     // Форматируем цену для лучшей читаемости (исправленная версия)
//     const priceStr = new BN(price.toString()).toString();
//     console.log("Token price:", priceStr, "tokens per LAMPORT");
//     // Также покажем цену за 1 SOL для удобства
//     const pricePerSol = Number(priceStr) * 1_000_000_000;
//     console.log("Token price:", pricePerSol.toLocaleString(), "tokens per SOL");
//     console.log("===============\n");
//   }

//   // Добавим функцию для вывода баланса пользователя
//   async function logUserBalance(message: string) {
//     const userSolBalance = await provider.connection.getBalance(
//       wallet.publicKey
//     );
//     let userTokenBalance;
//     try {
//       userTokenBalance = await provider.connection.getTokenAccountBalance(
//         userTokenAccount
//       );
//     } catch {
//       userTokenBalance = { value: { uiAmountString: "0" } };
//     }

//     console.log("\n=== " + message + " ===");
//     console.log(
//       "User SOL:",
//       (userSolBalance / anchor.web3.LAMPORTS_PER_SOL).toLocaleString(),
//       "SOL"
//     );
//     console.log(
//       "User tokens:",
//       Number(userTokenBalance.value.uiAmountString).toLocaleString()
//     );
//     console.log("===============\n");
//   }

//   before(async () => {
//     // Инициализируем все PDA и аккаунты перед тестами
//     mint = Keypair.generate();

//     const metadataProgramId = new PublicKey(
//       "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
//     );

//     [metadataPda] = PublicKey.findProgramAddressSync(
//       [
//         Buffer.from("metadata"),
//         metadataProgramId.toBuffer(),
//         mint.publicKey.toBuffer(),
//       ],
//       metadataProgramId
//     );

//     [poolAuthorityPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("pool_authority")],
//       liquidityPoolProgram.programId
//     );

//     [solAccountPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("sol_account"), mint.publicKey.toBuffer()],
//       liquidityPoolProgram.programId
//     );

//     poolTokenAccount = await getAssociatedTokenAddress(
//       mint.publicKey,
//       poolAuthorityPda,
//       true
//     );

//     userTokenAccount = await getAssociatedTokenAddress(
//       mint.publicKey,
//       wallet.publicKey
//     );

//     [mintSchedulePda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("mint_schedule"), mint.publicKey.toBuffer()],
//       program.programId
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

//   it("Creates token and initializes liquidity pool via CPI", async () => {
//     const tx = await program.methods
//       .createToken(
//         "One-Click Token",
//         "ONE",
//         "https://oneclick.com/token.json",
//         TOKEN_DECIMALS
//       )
//       .accounts({
//         mint: mint.publicKey,
//         metadata: metadataPda,
//         mintSchedule: mintSchedulePda,
//         authority: wallet.publicKey,
//         poolAccount: poolTokenAccount,
//         solAccount: solAccountPda,
//         poolAuthority: poolAuthorityPda,
//         liquidityPoolProgram: liquidityPoolProgram.programId,
//         tokenProgram: TOKEN_PROGRAM_ID,
//         associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//         systemProgram: anchor.web3.SystemProgram.programId,
//         rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//         tokenMetadataProgram: new PublicKey(
//           "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
//         ),
//       })
//       .signers([mint])
//       .rpc();

//     console.log("Create token tx:", tx);
//     await logPoolState("Initial pool state after creation");

//     // Проверяем балансы после создания
//     const poolTokenBalance = await provider.connection.getTokenAccountBalance(
//       poolTokenAccount
//     );

//     const solBalance = await provider.connection.getBalance(solAccountPda);

//     assert.strictEqual(
//       poolTokenBalance.value.uiAmount,
//       108_000_000,
//       "Pool should have initial minted tokens"
//     );

//     assert.approximately(
//       solBalance,
//       500_000_000,
//       1_000_000,
//       "Pool should have approximately 0.5 SOL initial balance (excluding rent)"
//     );
//   });

//   it("Allows swapping SOL for tokens and back", async () => {
//     // Получаем bump для pool_authority
//     const [poolAuthorityPda, poolAuthorityBump] =
//       PublicKey.findProgramAddressSync(
//         [Buffer.from("pool_authority")],
//         liquidityPoolProgram.programId
//       );

//     // Создаем ATA для пользователя, если его еще нет
//     const createAtaIx = createAssociatedTokenAccountInstruction(
//       wallet.publicKey, // payer
//       userTokenAccount, // ata
//       wallet.publicKey, // owner
//       mint.publicKey // mint
//     );

//     try {
//       await provider.sendAndConfirm(new Transaction().add(createAtaIx));
//       console.log("Created user ATA:", userTokenAccount.toString());
//     } catch (e) {
//       console.log("ATA already exists");
//     }

//     // Логируем начальное состояние
//     await logUserBalance("Initial user state");
//     await logPoolState("Initial pool state");

//     // Получаем начальную цену
//     const initialPrice = await liquidityPoolProgram.methods
//       .getPrice()
//       .accounts({
//         poolAccount: poolTokenAccount,
//         solAccount: solAccountPda,
//       })
//       .view();

//     console.log("Initial price:", initialPrice.toString());

//     // Покупаем токены за 0.1 SOL
//     const swapAmount = new BN(100_000_000); // 0.1 SOL
//     console.log(
//       "\nSwapping",
//       swapAmount.toNumber() / 1_000_000_000,
//       "SOL to tokens"
//     );

//     const swapTx = await liquidityPoolProgram.methods
//       .swapSolToTokens(swapAmount)
//       .accounts({
//         user: wallet.publicKey,
//         userTokenAccount: userTokenAccount,
//         poolAccount: poolTokenAccount,
//         solAccount: solAccountPda,
//         poolAuthority: poolAuthorityPda,
//         tokenProgram: TOKEN_PROGRAM_ID,
//         systemProgram: anchor.web3.SystemProgram.programId,
//       })
//       .signers([wallet.payer])
//       .preInstructions([
//         anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
//           units: 300_000,
//         }),
//       ])
//       .rpc();

//     console.log("Swap SOL to tokens tx:", swapTx);

//     // Логируем состояние после покупки
//     await logUserBalance("User state after buying tokens");
//     await logPoolState("Pool state after buying tokens");

//     // Получаем баланс для расчета продажи
//     const userTokenBalance = await provider.connection.getTokenAccountBalance(
//       userTokenAccount
//     );

//     // Продаем половину полученных токенов обратно
//     const sellAmount = new BN(userTokenBalance.value.amount).divn(2);
//     console.log(
//       "\nSelling",
//       (Number(sellAmount.toString()) / 1_000_000_000).toLocaleString(),
//       "tokens back to pool"
//     );

//     const sellTx = await liquidityPoolProgram.methods
//       .swapTokensToSol(sellAmount)
//       .accounts({
//         user: wallet.publicKey,
//         userTokenAccount: userTokenAccount,
//         poolAccount: poolTokenAccount,
//         solAccount: solAccountPda,
//         mint: mint.publicKey,
//         poolAuthority: poolAuthorityPda,
//         tokenProgram: TOKEN_PROGRAM_ID,
//       })
//       .rpc();

//     console.log("Swap tokens to SOL tx:", sellTx);

//     // Логируем финальное состояние
//     await logUserBalance("Final user state");
//     await logPoolState("Final pool state");

//     // Получаем финальные балансы для проверок
//     const finalUserTokenBalance =
//       await provider.connection.getTokenAccountBalance(userTokenAccount);
//     const finalPoolTokenBalance =
//       await provider.connection.getTokenAccountBalance(poolTokenAccount);
//     const finalSolBalance = await provider.connection.getBalance(solAccountPda);

//     // Исправленные проверки
//     assert(
//       Number(finalUserTokenBalance.value.amount) <
//         Number(userTokenBalance.value.amount),
//       "User tokens should decrease after selling"
//     );
//     assert(
//       Number(finalPoolTokenBalance.value.amount) > 108_000_000,
//       "Pool tokens should increase after user sells tokens back"
//     );
//     assert(
//       finalSolBalance < 600_000_000, // меньше 0.6 SOL
//       "Pool SOL should decrease after sending SOL to user"
//     );
//   });
// });
