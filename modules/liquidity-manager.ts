import * as anchor from "@coral-xyz/anchor";
import { AnchorProvider, Program } from "@coral-xyz/anchor";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction,
  createMintToInstruction,
} from "@solana/spl-token";
import BN from "bn.js";

interface LiquidityManagerResult {
  liquidityManagerAccount: PublicKey;
  poolSolAccount: PublicKey;
  poolNDollarAccount: PublicKey;
}

/**
 * Initializes Liquidity Manager and creates a liquidity pool
 */
export async function initializeLiquidityManager(
  provider: AnchorProvider,
  liquidityManagerProgram: Program,
  admin: Keypair,
  nDollarMint: PublicKey,
  nDollarDecimals: number,
  adminNDollarAccount: PublicKey
): Promise<LiquidityManagerResult> {
  try {
    // Find PDA for liquidity manager
    const [liquidityManagerPDA, liquidityManagerBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("liquidity_manager"), admin.publicKey.toBytes()],
        liquidityManagerProgram.programId
      );
    const liquidityManagerAccount = liquidityManagerPDA;

    // Create PDA for SOL pool storage
    const [poolSolPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool_sol"), liquidityManagerPDA.toBytes()],
      liquidityManagerProgram.programId
    );
    const poolSolAccount = poolSolPDA;

    // Create account for N-Dollar pool
    const poolNDollarAccount = await getAssociatedTokenAddress(
      nDollarMint,
      liquidityManagerAccount,
      true
    );

    // Initialize liquidity manager
    await liquidityManagerProgram.methods
      .initializeLiquidityManager()
      .accounts({
        authority: admin.publicKey,
        nDollarMint: nDollarMint,
        liquidityManager: liquidityManagerAccount,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([admin])
      .rpc();

    // Request SOL for the liquidity pool
    const createPoolSolAccountTx = await provider.connection.requestAirdrop(
      poolSolAccount,
      5 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(createPoolSolAccountTx);

    // Create N-Dollar pool account
    const tx = new Transaction();
    tx.add(
      createAssociatedTokenAccountInstruction(
        admin.publicKey,
        poolNDollarAccount,
        liquidityManagerAccount,
        nDollarMint
      )
    );

    // Send transaction
    await anchor.web3.sendAndConfirmTransaction(provider.connection, tx, [
      admin,
    ]);

    // Add liquidity to the pool (5 SOL and almost all N-Dollar for sale availability)
    const solAmount = new BN(5 * anchor.web3.LAMPORTS_PER_SOL);

    // Transfer N-Dollar to the pool in batches
    // First transfer the first portion (50 million)
    const batch1 = new BN(50_000_000).mul(
      new BN(10).pow(new BN(nDollarDecimals))
    );
    const transferTx1 = new Transaction();
    transferTx1.add(
      createMintToInstruction(
        nDollarMint,
        poolNDollarAccount,
        admin.publicKey,
        Number(batch1.toString())
      )
    );
    await anchor.web3.sendAndConfirmTransaction(
      provider.connection,
      transferTx1,
      [admin]
    );

    // Then the rest (approximately 56.9 million, to reach ~99% of 108 million)
    const batch2 = new BN(56_900_000).mul(
      new BN(10).pow(new BN(nDollarDecimals))
    );
    const transferTx2 = new Transaction();
    transferTx2.add(
      createMintToInstruction(
        nDollarMint,
        poolNDollarAccount,
        admin.publicKey,
        Number(batch2.toString())
      )
    );
    await anchor.web3.sendAndConfirmTransaction(
      provider.connection,
      transferTx2,
      [admin]
    );

    // Add SOL to the pool through add_liquidity command
    await liquidityManagerProgram.methods
      .addLiquidity(
        solAmount,
        new BN(0) // we've already added N-Dollar directly
      )
      .accounts({
        authority: admin.publicKey,
        liquidityManager: liquidityManagerAccount,
        authorityNdollarAccount: adminNDollarAccount,
        poolSolAccount: poolSolAccount,
        poolNdollarAccount: poolNDollarAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin])
      .rpc();

    // Check pool balances
    const poolSolBalance = await provider.connection.getBalance(poolSolAccount);
    const poolNDollarBalance = await provider.connection.getTokenAccountBalance(
      poolNDollarAccount
    );

    console.log(
      "SOL balance in pool:",
      poolSolBalance / anchor.web3.LAMPORTS_PER_SOL,
      "SOL"
    );
    console.log(
      "N-Dollar balance in pool:",
      poolNDollarBalance.value.uiAmount,
      "NDOL"
    );

    return {
      liquidityManagerAccount,
      poolSolAccount,
      poolNDollarAccount,
    };
  } catch (error) {
    console.log("Error initializing Liquidity Manager:", error);
    throw error;
  }
}
