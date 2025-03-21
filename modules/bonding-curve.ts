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
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction,
  createInitializeMintInstruction,
  createMintToInstruction,
  createSetAuthorityInstruction,
  AuthorityType,
} from "@solana/spl-token";
import { assert } from "chai";
import BN from "bn.js";

interface MemeCoinResult {
  memeCoinMint: PublicKey;
  adminMemeCoinAccount: PublicKey;
  user1MemeCoinAccount: PublicKey;
  user2MemeCoinAccount: PublicKey;
  liquidity_pool: PublicKey;
  bondingCurveAccount: PublicKey;
}

/**
 * Creates a memecoin and sets up a bonding curve
 */
export async function createMemeCoinWithBondingCurve(
  provider: AnchorProvider,
  bondingCurveProgram: Program,
  admin: Keypair,
  user1: Keypair,
  user2: Keypair,
  nDollarMint: PublicKey,
  nDollarDecimals: number
): Promise<MemeCoinResult> {
  // Create mint for memecoin
  const memeCoinMintKeypair = Keypair.generate();
  const memeCoinMint = memeCoinMintKeypair.publicKey;

  // Create associated token accounts for all users
  const adminMemeCoinAccount = await getAssociatedTokenAddress(
    memeCoinMint,
    admin.publicKey
  );

  const user1MemeCoinAccount = await getAssociatedTokenAddress(
    memeCoinMint,
    user1.publicKey
  );

  const user2MemeCoinAccount = await getAssociatedTokenAddress(
    memeCoinMint,
    user2.publicKey
  );

  // Create separate N-Dollar liquidity pool for bonding curve
  const [bondingCurvePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("bonding_curve"), memeCoinMint.toBytes()],
    bondingCurveProgram.programId
  );
  const bondingCurveAccount = bondingCurvePDA;

  // Create account for liquidity pool owned by bonding curve
  const liquidity_pool = await getAssociatedTokenAddress(
    nDollarMint,
    bondingCurveAccount,
    true
  );

  // Create transaction for mint initialization and associated token account creation
  const tx = new Transaction();

  // Initialize mint
  tx.add(
    SystemProgram.createAccount({
      fromPubkey: admin.publicKey,
      newAccountPubkey: memeCoinMint,
      lamports: await provider.connection.getMinimumBalanceForRentExemption(82),
      space: 82,
      programId: TOKEN_PROGRAM_ID,
    })
  );

  tx.add(
    createInitializeMintInstruction(
      memeCoinMint,
      9, // 9 decimals
      admin.publicKey,
      admin.publicKey
    )
  );

  // Create associated token accounts
  tx.add(
    createAssociatedTokenAccountInstruction(
      admin.publicKey,
      adminMemeCoinAccount,
      admin.publicKey,
      memeCoinMint
    )
  );

  tx.add(
    createAssociatedTokenAccountInstruction(
      admin.publicKey,
      user1MemeCoinAccount,
      user1.publicKey,
      memeCoinMint
    )
  );

  tx.add(
    createAssociatedTokenAccountInstruction(
      admin.publicKey,
      user2MemeCoinAccount,
      user2.publicKey,
      memeCoinMint
    )
  );

  // Send transaction
  await anchor.web3.sendAndConfirmTransaction(provider.connection, tx, [
    admin,
    memeCoinMintKeypair,
  ]);

  // Create liquidity pool for bonding curve
  const liquidityPoolTx = new Transaction();
  liquidityPoolTx.add(
    createAssociatedTokenAccountInstruction(
      admin.publicKey,
      liquidity_pool,
      bondingCurveAccount,
      nDollarMint
    )
  );

  await anchor.web3.sendAndConfirmTransaction(
    provider.connection,
    liquidityPoolTx,
    [admin]
  );

  // Mint initial liquidity to pool
  const mintToPoolTx = new Transaction();
  const poolAmount = new BN(1000 * Math.pow(10, nDollarDecimals)); // 1000 N-Dollar for liquidity

  mintToPoolTx.add(
    createMintToInstruction(
      nDollarMint,
      liquidity_pool,
      admin.publicKey,
      poolAmount.toNumber()
    )
  );

  await anchor.web3.sendAndConfirmTransaction(
    provider.connection,
    mintToPoolTx,
    [admin]
  );

  console.log("Liquidity pool for bonding curve created with 1000 N-Dollar");

  // Set initial bonding curve parameters
  const initialPrice = new BN(1000000); // 0.001 N-Dollar
  const power = 2; // Power exponent for curve
  const feePercent = 100; // 1% fee (in basis points)

  // Initialize bonding curve
  await bondingCurveProgram.methods
    .initializeBondingCurve(memeCoinMint, initialPrice, power, feePercent)
    .accounts({
      creator: admin.publicKey,
      bondingCurve: bondingCurveAccount,
      coinMint: memeCoinMint,
      ndollarMint: nDollarMint,
      liquidityPool: liquidity_pool, // Use new liquidity pool
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
    })
    .signers([admin])
    .rpc();

  // Transfer mint authority rights to bonding-curve contract
  const transferAuthorityTx = new Transaction();
  transferAuthorityTx.add(
    createSetAuthorityInstruction(
      memeCoinMint,
      admin.publicKey,
      AuthorityType.MintTokens,
      bondingCurveAccount
    )
  );

  await anchor.web3.sendAndConfirmTransaction(
    provider.connection,
    transferAuthorityTx,
    [admin]
  );

  console.log("Memecoin successfully created:", memeCoinMint.toString());
  console.log("Bonding curve established:", bondingCurveAccount.toString());
  console.log("Mint authority rights transferred to bonding curve");
  console.log("Liquidity pool:", liquidity_pool.toString());

  return {
    memeCoinMint,
    adminMemeCoinAccount,
    user1MemeCoinAccount,
    user2MemeCoinAccount,
    liquidity_pool,
    bondingCurveAccount,
  };
}

/**
 * Helper function to get current memecoin price
 */
async function getMemecoinPrice(
  provider: AnchorProvider,
  bondingCurveProgram: Program,
  bondingCurveAccount: PublicKey,
  memeCoinMint: PublicKey
): Promise<number> {
  try {
    // Get bonding curve account data
    const bondingCurveInfo = await provider.connection.getAccountInfo(
      bondingCurveAccount
    );
    if (!bondingCurveInfo) {
      console.log("Bonding curve account not found");
      return 0;
    }

    // Deserialize account data
    const accountData = bondingCurveProgram.coder.accounts.decode(
      "bondingCurve",
      bondingCurveInfo.data
    );

    // Get necessary values for price calculation
    const totalSupply = accountData.totalSupplyInCurve;
    const reserveBalance = accountData.reserveBalance;
    const power = accountData.power;
    const initialPrice = accountData.initialPrice;

    // If supply = 0, return initial price
    if (totalSupply.isZero()) {
      // Initial price in structure is stored in N-Dollar lamports
      const priceInNDollar = initialPrice.toNumber() / Math.pow(10, 9);
      console.log(
        `Current memecoin price: ${priceInNDollar} N-Dollar (initial price)`
      );
      return priceInNDollar;
    }

    // Use RPC method to get current price
    try {
      // Log data for price calculation
      console.log(
        `Total Supply: ${totalSupply.toString()}, Reserve Balance: ${reserveBalance.toString()}, Power: ${power}`
      );

      // Calculate price using formula: price = (reserve_balance * power) / total_supply
      const price = reserveBalance.mul(new BN(power)).div(totalSupply);
      const priceInNDollar = price.toNumber() / Math.pow(10, 9);

      console.log(`Current memecoin price: ${priceInNDollar} N-Dollar`);
      return priceInNDollar;
    } catch (error) {
      console.log("Error calculating price:", error.message);
      return 0;
    }
  } catch (error) {
    console.log("Error getting memecoin price:", error.message);
    return 0;
  }
}

/**
 * Buys and sells memecoin using bonding curve
 */
export async function tradeMemeCoin(
  provider: AnchorProvider,
  bondingCurveProgram: Program,
  admin: Keypair,
  user: Keypair,
  nDollarMint: PublicKey,
  memeCoinMint: PublicKey,
  userNDollarAccount: PublicKey,
  userMemeCoinAccount: PublicKey,
  bondingCurveAccount: PublicKey,
  liquidity_pool: PublicKey,
  nDollarDecimals: number
): Promise<void> {
  try {
    // Get initial balances
    const initialNDollarBalance =
      await provider.connection.getTokenAccountBalance(userNDollarAccount);
    const initialMemeCoinBalance =
      await provider.connection.getTokenAccountBalance(userMemeCoinAccount);

    console.log(
      "Initial N-Dollar balance:",
      initialNDollarBalance.value.uiAmount,
      "NDOL"
    );
    console.log(
      "Initial memecoin balance:",
      initialMemeCoinBalance.value.uiAmount
    );

    // First transfer N-Dollar to user
    const ndollarToUserTx = new Transaction();
    const transferAmount = new BN(100 * Math.pow(10, nDollarDecimals)); // 100 N-Dollar

    ndollarToUserTx.add(
      createMintToInstruction(
        nDollarMint,
        userNDollarAccount,
        admin.publicKey,
        transferAmount.toNumber()
      )
    );

    await anchor.web3.sendAndConfirmTransaction(
      provider.connection,
      ndollarToUserTx,
      [admin]
    );

    console.log("User granted 100 N-Dollar for buying memecoins");

    // Test amounts array (in N-Dollar)
    const testAmounts = [
      0.00000001, // Extremely small amount
      0.001, // Small amount
      1, // Medium amount
      10, // Large amount
      50, // Very large amount
    ];

    // Test each amount
    for (const amount of testAmounts) {
      const ndollarAmount = new BN(
        Math.floor(amount * Math.pow(10, nDollarDecimals))
      );

      console.log(`\nAttempting to buy memecoins for ${amount} N-Dollar`);

      // Get price before purchase
      console.log(`Memecoin price BEFORE buying with ${amount} N-Dollar:`);
      await getMemecoinPrice(
        provider,
        bondingCurveProgram,
        bondingCurveAccount,
        memeCoinMint
      );

      try {
        await bondingCurveProgram.methods
          .buyToken(ndollarAmount)
          .accounts({
            buyer: user.publicKey,
            bondingCurve: bondingCurveAccount,
            coinMint: memeCoinMint,
            ndollarMint: nDollarMint,
            buyerCoinAccount: userMemeCoinAccount,
            buyerNdollarAccount: userNDollarAccount,
            liquidityPool: liquidity_pool,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([user])
          .rpc();

        // Get new balances after this purchase
        const newBalance = await provider.connection.getTokenAccountBalance(
          userMemeCoinAccount
        );

        console.log(
          `Successfully purchased! Current memecoin balance: ${newBalance.value.uiAmount}`
        );

        // Get price after purchase
        console.log(`Memecoin price AFTER buying with ${amount} N-Dollar:`);
        await getMemecoinPrice(
          provider,
          bondingCurveProgram,
          bondingCurveAccount,
          memeCoinMint
        );
      } catch (error) {
        console.log(`Error buying with ${amount} N-Dollar:`, error.message);
      }
    }

    // Get final balances
    const finalNDollarBalance =
      await provider.connection.getTokenAccountBalance(userNDollarAccount);
    const finalMemeCoinBalance =
      await provider.connection.getTokenAccountBalance(userMemeCoinAccount);

    console.log(
      "\nFinal N-Dollar balance:",
      finalNDollarBalance.value.uiAmount,
      "NDOL"
    );
    console.log("Final memecoin balance:", finalMemeCoinBalance.value.uiAmount);

    // Test selling memecoin
    if (finalMemeCoinBalance.value.uiAmount > 0) {
      // Calculate different amounts for selling
      const totalAmount = parseInt(finalMemeCoinBalance.value.amount);

      // Percentage array for sale testing
      const percentages = [
        0.0000001, // Extremely small percentage
        0.001, // Small percentage
        0.01, // 1%
        0.1, // 10%
        0.5, // 50%
      ];

      for (const percentage of percentages) {
        const tokenAmountToSell = Math.floor(totalAmount * percentage);
        if (tokenAmountToSell <= 0) continue;

        const amount = new BN(tokenAmountToSell);

        console.log(
          `\nAttempting to sell ${
            tokenAmountToSell / Math.pow(10, 9)
          } memecoins (${percentage * 100}% of total)`
        );

        // Get price before sale
        console.log(
          `Memecoin price BEFORE selling ${percentage * 100}% of tokens:`
        );
        await getMemecoinPrice(
          provider,
          bondingCurveProgram,
          bondingCurveAccount,
          memeCoinMint
        );

        try {
          await bondingCurveProgram.methods
            .sellToken(amount)
            .accounts({
              buyer: user.publicKey,
              bondingCurve: bondingCurveAccount,
              coinMint: memeCoinMint,
              ndollarMint: nDollarMint,
              buyerCoinAccount: userMemeCoinAccount,
              buyerNdollarAccount: userNDollarAccount,
              liquidityPool: liquidity_pool,
              tokenProgram: TOKEN_PROGRAM_ID,
              systemProgram: SystemProgram.programId,
            })
            .signers([user])
            .rpc();

          const newBalance = await provider.connection.getTokenAccountBalance(
            userMemeCoinAccount
          );

          console.log(
            `Successfully sold! Current memecoin balance: ${newBalance.value.uiAmount}`
          );

          // Get price after sale
          console.log(
            `Memecoin price AFTER selling ${percentage * 100}% of tokens:`
          );
          await getMemecoinPrice(
            provider,
            bondingCurveProgram,
            bondingCurveAccount,
            memeCoinMint
          );
        } catch (error) {
          console.log(
            `Error selling ${tokenAmountToSell / Math.pow(10, 9)} memecoins:`,
            error.message
          );
        }
      }
    } else {
      console.log("User has no memecoins to sell");
    }
  } catch (error) {
    console.log("Error trading memecoin:", error);
  }
}

/**
 * Simulates memecoin purchase considering slippage for different amounts
 */
export async function simulateMemeCoinPricing(
  bondingCurveProgram: Program,
  bondingCurveAccount: PublicKey,
  memeCoinMint: PublicKey,
  nDollarDecimals: number
): Promise<void> {
  try {
    // Test different amounts for simulation
    const testAmounts = [
      0.00000001, // Extremely small amount
      0.001, // Small amount
      1, // Medium amount
      10, // Large amount
      1000, // Extremely large amount
      1000000, // Massively large amount
    ];

    console.log("\nSimulation of memecoin purchases for different amounts:");

    for (const amount of testAmounts) {
      const ndollarAmount = new BN(
        Math.floor(amount * Math.pow(10, nDollarDecimals))
      );

      console.log(`\nSimulation for buying with ${amount} N-Dollar:`);

      try {
        // Call simulate_buy method to get price info considering slippage
        await bondingCurveProgram.methods
          .simulateBuy(ndollarAmount)
          .accounts({
            bondingCurve: bondingCurveAccount,
            coinMint: memeCoinMint,
          })
          .rpc();

        console.log(`Simulation for ${amount} N-Dollar successfully completed`);
      } catch (error) {
        console.log(
          `Error in simulation for ${amount} N-Dollar:`,
          error.message
        );
      }
    }

    // Get current token price
    await bondingCurveProgram.methods
      .calculatePrice()
      .accounts({
        bondingCurve: bondingCurveAccount,
        coinMint: memeCoinMint,
      })
      .rpc();
  } catch (error) {
    console.log("General error in memecoin purchase simulation:", error);
  }
}
