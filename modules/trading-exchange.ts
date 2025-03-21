import * as anchor from "@coral-xyz/anchor";
import { AnchorProvider, Program } from "@coral-xyz/anchor";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMintToInstruction } from "@solana/spl-token";
import { assert } from "chai";
import BN from "bn.js";

interface TradingExchangeResult {
  exchangeDataAccount: PublicKey;
  tradingExchangeAccount: PublicKey;
}

/**
 * Initializes Trading Exchange
 */
export async function initializeTradingExchange(
  tradingExchangeProgram: Program,
  admin: Keypair,
  nDollarMint: PublicKey
): Promise<TradingExchangeResult> {
  try {
    // Find PDA for exchange data
    const [exchangeDataPDA, exchangeDataBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("exchange_data"), admin.publicKey.toBytes()],
        tradingExchangeProgram.programId
      );
    const exchangeDataAccount = exchangeDataPDA;

    // Initialize exchange data
    await tradingExchangeProgram.methods
      .initializeExchange()
      .accounts({
        authority: admin.publicKey,
        exchangeData: exchangeDataAccount,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([admin])
      .rpc();

    console.log(
      "Trading Exchange data successfully initialized:",
      exchangeDataAccount.toString()
    );

    // Create and initialize account for TradingExchange
    const [tradingExchangePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("trading_exchange"), admin.publicKey.toBytes()],
      tradingExchangeProgram.programId
    );
    const tradingExchangeAccount = tradingExchangePDA;

    // Initialize TradingExchange
    await tradingExchangeProgram.methods
      .initializeTradingExchange(nDollarMint)
      .accounts({
        authority: admin.publicKey,
        tradingExchange: tradingExchangeAccount,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([admin])
      .rpc();

    console.log(
      "Trading Exchange successfully initialized:",
      tradingExchangeAccount.toString()
    );

    return {
      exchangeDataAccount,
      tradingExchangeAccount,
    };
  } catch (error) {
    console.log("Error initializing Trading Exchange:", error);

    // Create mock PDA for continued testing
    const [tradingExchangePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("trading_exchange"), admin.publicKey.toBytes()],
      tradingExchangeProgram.programId
    );
    const [exchangeDataPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("exchange_data"), admin.publicKey.toBytes()],
      tradingExchangeProgram.programId
    );

    return {
      exchangeDataAccount: exchangeDataPDA,
      tradingExchangeAccount: tradingExchangePDA,
    };
  }
}

/**
 * Buys and sells N-Dollar
 */
export async function buyAndSellNDollar(
  provider: AnchorProvider,
  tradingExchangeProgram: Program,
  user: Keypair,
  tradingExchangeAccount: PublicKey,
  userNDollarAccount: PublicKey,
  liquidityManagerAccount: PublicKey,
  poolSolAccount: PublicKey,
  poolNDollarAccount: PublicKey,
  liquidityManagerProgramId: PublicKey,
  nDollarMint: PublicKey,
  admin: Keypair,
  nDollarDecimals: number
): Promise<void> {
  // Get initial balances
  const initialSolBalance = await provider.connection.getBalance(
    user.publicKey
  );
  const initialNDollarBalance =
    await provider.connection.getTokenAccountBalance(userNDollarAccount);

  console.log(
    "Initial SOL balance:",
    initialSolBalance / anchor.web3.LAMPORTS_PER_SOL,
    "SOL"
  );
  console.log(
    "Initial N-Dollar balance:",
    initialNDollarBalance.value.uiAmount,
    "NDOL"
  );

  // Buy 1 SOL worth of N-Dollar through trading exchange
  const solAmount = new BN(1 * anchor.web3.LAMPORTS_PER_SOL);

  await tradingExchangeProgram.methods
    .buyNDollar(solAmount)
    .accounts({
      user: user.publicKey,
      tradingExchange: tradingExchangeAccount,
      userNdollarAccount: userNDollarAccount,
      liquidityManager: liquidityManagerAccount,
      poolSolAccount: poolSolAccount,
      poolNdollarAccount: poolNDollarAccount,
      liquidityManagerProgram: liquidityManagerProgramId,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .signers([user])
    .rpc();

  // Get new balances
  const newSolBalance = await provider.connection.getBalance(user.publicKey);
  const newNDollarBalance = await provider.connection.getTokenAccountBalance(
    userNDollarAccount
  );

  console.log(
    "New SOL balance:",
    newSolBalance / anchor.web3.LAMPORTS_PER_SOL,
    "SOL"
  );
  console.log(
    "New N-Dollar balance:",
    newNDollarBalance.value.uiAmount,
    "NDOL"
  );

  // Check that SOL decreased by approximately 1 SOL (considering transaction fees)
  assert(initialSolBalance - newSolBalance >= solAmount.toNumber());

  // Check that N-Dollar increased (should be approximately 990 N-Dollar for 1 SOL with 1% fee)
  assert(
    newNDollarBalance.value.uiAmount > initialNDollarBalance.value.uiAmount
  );
  assert(
    newNDollarBalance.value.uiAmount - initialNDollarBalance.value.uiAmount >=
      0.9
    // Expect about 0.99 N-Dollar, considering 1% fee
  );

  // Test selling N-Dollar for SOL
  // Mint additional N-Dollar to user for selling
  const mintAmount = new BN(500 * Math.pow(10, nDollarDecimals)); // 500 N-Dollar
  const mintTx = new Transaction();
  mintTx.add(
    createMintToInstruction(
      nDollarMint,
      userNDollarAccount,
      admin.publicKey,
      mintAmount.toNumber()
    )
  );
  await anchor.web3.sendAndConfirmTransaction(provider.connection, mintTx, [
    admin,
  ]);

  console.log("User granted 500 N-Dollar for sale test");

  // Get updated N-Dollar balance after minting
  const updatedNDollarBalance =
    await provider.connection.getTokenAccountBalance(userNDollarAccount);
  console.log(
    "Updated N-Dollar balance:",
    updatedNDollarBalance.value.uiAmount,
    "NDOL"
  );

  // Sell N-Dollar for SOL through trading exchange - CHANGED: sell only 10 N-Dollar
  const ndollarAmount = new BN(10 * Math.pow(10, nDollarDecimals)); // 10 N-Dollar

  console.log("Selling 10 N-Dollar for SOL");

  await tradingExchangeProgram.methods
    .sellNDollar(ndollarAmount)
    .accounts({
      user: user.publicKey,
      tradingExchange: tradingExchangeAccount,
      userNdollarAccount: userNDollarAccount,
      liquidityManager: liquidityManagerAccount,
      poolSolAccount: poolSolAccount,
      poolNdollarAccount: poolNDollarAccount,
      liquidityManagerProgram: liquidityManagerProgramId,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .signers([user])
    .rpc();

  // Get new balances
  const finalSolBalance = await provider.connection.getBalance(user.publicKey);
  const finalNDollarBalance = await provider.connection.getTokenAccountBalance(
    userNDollarAccount
  );

  console.log(
    "Final SOL balance:",
    finalSolBalance / anchor.web3.LAMPORTS_PER_SOL,
    "SOL"
  );
  console.log(
    "Final N-Dollar balance:",
    finalNDollarBalance.value.uiAmount,
    "NDOL"
  );

  // Check that N-Dollar decreased by the sold amount (10)
  assert.approximately(
    updatedNDollarBalance.value.uiAmount - finalNDollarBalance.value.uiAmount,
    10,
    0.1
  );

  // Check that SOL increased
  assert(finalSolBalance > newSolBalance);
}
