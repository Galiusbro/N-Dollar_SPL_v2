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
} from "@solana/spl-token";
import { assert } from "chai";
import BN from "bn.js";

/**
 * Tests N-Dollar pricing during buy and sell operations
 */
export async function testPricing(
  provider: AnchorProvider,
  tradingExchangeProgram: Program,
  liquidityManagerProgram: Program,
  nDollarMint: PublicKey,
  tradingExchangeAccount: PublicKey,
  liquidityManagerAccount: PublicKey,
  poolSolAccount: PublicKey,
  poolNDollarAccount: PublicKey,
  nDollarDecimals: number
): Promise<void> {
  // Function to get current price from Liquidity Manager account
  async function getCurrentPrice(): Promise<number> {
    const liquidityManagerInfo = await provider.connection.getAccountInfo(
      liquidityManagerAccount
    );
    if (!liquidityManagerInfo) {
      throw new Error("Liquidity Manager account not found");
    }
    // Decode account data - currentPrice field is after:
    // - 8 bytes discriminator
    // - 32 bytes authority
    // - 32 bytes n_dollar_mint
    // - 8 bytes total_liquidity
    // - 8 bytes total_users
    // = 88 bytes before currentPrice field, which occupies 8 bytes
    const currentPriceOffset = 8 + 32 + 32 + 8 + 8;
    const currentPrice = new BN(
      liquidityManagerInfo.data.slice(
        currentPriceOffset,
        currentPriceOffset + 8
      ),
      "le"
    ).toNumber();

    // current_price is the amount of N-Dollar per 1 SOL (in base units)
    const priceInNDollarPerSol = currentPrice / Math.pow(10, nDollarDecimals);
    return priceInNDollarPerSol;
  }

  // Check initial price
  const initialPrice = await getCurrentPrice();
  console.log("Initial price: 1 SOL =", initialPrice, "N-Dollar");

  // Create new user for this test
  const testUser = Keypair.generate();
  await provider.connection.requestAirdrop(
    testUser.publicKey,
    3 * anchor.web3.LAMPORTS_PER_SOL
  );
  await new Promise((resolve) => setTimeout(resolve, 1000)); // Wait for confirmation

  // Create token account for test user
  const testUserNDollarAccount = await getAssociatedTokenAddress(
    nDollarMint,
    testUser.publicKey
  );

  const setupTx = new Transaction();
  setupTx.add(
    createAssociatedTokenAccountInstruction(
      testUser.publicKey,
      testUserNDollarAccount,
      testUser.publicKey,
      nDollarMint
    )
  );
  await anchor.web3.sendAndConfirmTransaction(provider.connection, setupTx, [
    testUser,
  ]);

  // Buy N-Dollar for 1 SOL
  const solAmount = new BN(1 * anchor.web3.LAMPORTS_PER_SOL);

  console.log("Buying N-Dollar for 1 SOL...");
  await tradingExchangeProgram.methods
    .buyNDollar(solAmount)
    .accounts({
      user: testUser.publicKey,
      tradingExchange: tradingExchangeAccount,
      userNdollarAccount: testUserNDollarAccount,
      liquidityManager: liquidityManagerAccount,
      poolSolAccount: poolSolAccount,
      poolNdollarAccount: poolNDollarAccount,
      liquidityManagerProgram: liquidityManagerProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .signers([testUser])
    .rpc();

  // Get amount of purchased N-Dollar
  const boughtNDollarBalance = await provider.connection.getTokenAccountBalance(
    testUserNDollarAccount
  );
  console.log(
    "Bought:",
    boughtNDollarBalance.value.uiAmount,
    "N-Dollar for 1 SOL"
  );

  // Check price after purchase
  const priceAfterBuy = await getCurrentPrice();
  console.log("Price after purchase: 1 SOL =", priceAfterBuy, "N-Dollar");
  console.log(
    "Price change after purchase:",
    (((priceAfterBuy - initialPrice) / initialPrice) * 100).toFixed(2),
    "%"
  );

  // Sell N-Dollar for SOL (half of purchased amount)
  const halfBoughtAmountBN = new BN(
    Math.floor(
      (boughtNDollarBalance.value.uiAmount! * Math.pow(10, nDollarDecimals)) / 2
    )
  );

  console.log(
    "Selling",
    halfBoughtAmountBN.toNumber() / Math.pow(10, nDollarDecimals),
    "N-Dollar for SOL..."
  );
  await tradingExchangeProgram.methods
    .sellNDollar(halfBoughtAmountBN)
    .accounts({
      user: testUser.publicKey,
      tradingExchange: tradingExchangeAccount,
      userNdollarAccount: testUserNDollarAccount,
      liquidityManager: liquidityManagerAccount,
      poolSolAccount: poolSolAccount,
      poolNdollarAccount: poolNDollarAccount,
      liquidityManagerProgram: liquidityManagerProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .signers([testUser])
    .rpc();

  // Get remaining N-Dollar amount
  const remainingNDollarBalance =
    await provider.connection.getTokenAccountBalance(testUserNDollarAccount);
  console.log("Remaining N-Dollar:", remainingNDollarBalance.value.uiAmount);
  console.log(
    "Sold N-Dollar:",
    boughtNDollarBalance.value.uiAmount! -
      remainingNDollarBalance.value.uiAmount!
  );

  // Check price after sale
  const priceAfterSell = await getCurrentPrice();
  console.log("Price after sale: 1 SOL =", priceAfterSell, "N-Dollar");
  console.log(
    "Price change after sale:",
    (((priceAfterSell - priceAfterBuy) / priceAfterBuy) * 100).toFixed(2),
    "%"
  );

  // Check overall price change
  console.log(
    "Overall price change:",
    (((priceAfterSell - initialPrice) / initialPrice) * 100).toFixed(2),
    "%"
  );

  // Check that price changed
  assert(priceAfterBuy !== initialPrice, "Price did not change after purchase");
  assert(priceAfterSell !== priceAfterBuy, "Price did not change after sale");
}
