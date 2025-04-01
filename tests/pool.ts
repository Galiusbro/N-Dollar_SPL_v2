// import { PublicKey } from "@solana/web3.js";

// // Заменить на твой токен
// const nDollarMint = new PublicKey(
//   "CFVQG5L1RV6HtVNqTzq43VJVutwUmrFC1FHXFaLhy1Ch"
// );

// // Это адрес программы LiquidityPool, обязательно тот же, с которым создавал
// const liquidityPoolProgramId = new PublicKey(
//   "B24yupzEDjF7Z9frnDG16uAwH1ZYfB57kuzh8jwDsL83"
// ); // <-- вставь свой ID!

// // Найти pool PDA
// const [poolPda] = PublicKey.findProgramAddressSync(
//   [Buffer.from("pool"), nDollarMint.toBuffer()],
//   liquidityPoolProgramId
// );

// console.log("📦 Liquidity Pool PDA:", poolPda.toBase58());

// // Дополнительно, можешь найти SOL Vault и N-Dollar ATA:
// const [solVaultPda] = PublicKey.findProgramAddressSync(
//   [Buffer.from("sol_vault"), poolPda.toBuffer()],
//   liquidityPoolProgramId
// );

// console.log("💰 SOL Vault PDA:", solVaultPda.toBase58());

// // Если надо — ATA для пула (vault) для N-Dollar:
// import { getAssociatedTokenAddressSync } from "@solana/spl-token";
// const poolNdollarAta = getAssociatedTokenAddressSync(
//   nDollarMint,
//   poolPda,
//   true
// );

// console.log("🎯 Pool N-Dollar ATA:", poolNdollarAta.toBase58());
