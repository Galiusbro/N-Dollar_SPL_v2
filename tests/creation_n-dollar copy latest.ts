import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  getAccount,
  createAssociatedTokenAccountInstruction,
  TokenAccountNotFoundError,
} from "@solana/spl-token";
import { assert } from "chai";
import { BN } from "bn.js";

// Helper function to safely get token balance (returns 0 if account doesn't exist)
async function getTokenBalance(
  provider: anchor.Provider,
  ata: PublicKey
): Promise<bigint> {
  try {
    const accountInfo = await getAccount(provider.connection, ata);
    return accountInfo.amount;
  } catch (error) {
    // --- ИЗМЕНЕНО: Проверка на конкретный тип ошибки ---
    if (error instanceof TokenAccountNotFoundError) {
      // Логируем, что аккаунт не найден (это ожидаемо в некоторых случаях)
      // console.log(`ATA ${ata.toString()} not found (expected in some cases). Returning balance 0.`);
      return BigInt(0);
    }
    // Можно оставить старые проверки на всякий случай, хотя instanceof должен быть надежнее
    else if (
      error.message.includes("could not find account") ||
      error.message.includes("Account does not exist")
    ) {
      // console.log(`ATA ${ata.toString()} does not exist or has 0 balance.`);
      return BigInt(0);
    }
    // Re-throw other unexpected errors
    console.error(
      `Unexpected error fetching account ${ata.toString()}:`,
      error
    );
    throw error;
  }
}

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

describe("Token Creation and Distribution Test", () => {
  // Program and account setup
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const wallet = provider.wallet as anchor.Wallet;

  // --- ИЗМЕНЕНО: Загрузка всех необходимых программ ---
  const tokenCreatorProgram = anchor.workspace.TokenCreator as Program;
  const tokenDistributorProgram = anchor.workspace.TokenDistributor as Program;
  const bondingCurveProgram = anchor.workspace.BondingCurve as Program;
  const liquidityPoolProgram = anchor.workspace.LiquidityPool as Program;
  const NdollarProgram = anchor.workspace.NDollar as Program;

  // Глобальные переменные для тестов
  let userTokenMint: PublicKey;
  let userTokenMintKp: Keypair;
  let bondingCurvePda: PublicKey;
  let nDollarTreasury: PublicKey;
  let bondingCurveAuthorityPda: PublicKey;
  let bondingCurveTokenAccount: PublicKey;
  let userTokenAccount: PublicKey;
  let distributorTokenAccount: PublicKey;
  let tokenInfoPda: PublicKey;
  let distributorAuthorityPda: PublicKey;

  const METADATA_PROGRAM_ID = new PublicKey(
    "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
  );
  const WRAPPED_SOL_MINT = new PublicKey(
    "So11111111111111111111111111111111111111112"
  );
  const SOL_DECIMALS = anchor.web3.LAMPORTS_PER_SOL;

  // N-Dollar Mint Keypair (оставляем как было)
  const nDollarMintKp = Keypair.generate();
  const nDollarMint = nDollarMintKp.publicKey;

  // PDAs и ATAs для N-Dollar и пула (оставляем как было)
  let nDollarMetadataPda: PublicKey;
  let poolPda: PublicKey;
  let solVaultPda: PublicKey;
  let poolNDollarAccount: PublicKey; // ATA пула для N-Dollar
  let userNDollarAccount: PublicKey; // ATA пользователя для N-Dollar

  // Utility functions (оставляем)
  // async function getOrCreateATA... (можно использовать getAssociatedTokenAddressSync + getAccount из spl-token)
  async function airdropSol(address: PublicKey, amount: number) {
    /* ... */
  }
  function readMetaplexString(
    buffer: Buffer,
    offset: number
  ): [string, number] {
    const length = buffer.readUInt32LE(offset);
    if (length === 0) return ["", offset + 4];

    const str = buffer.slice(offset + 4, offset + 4 + length).toString("utf8");
    return [str, offset + 4 + length];
  }

  function parseMetadata(data: Buffer) {
    try {
      let offset = 1 + 32 + 32; // Skip past header
      const [name, nameEnd] = readMetaplexString(data, offset);
      const [symbol, symbolEnd] = readMetaplexString(data, nameEnd);
      const [uri] = readMetaplexString(data, symbolEnd);

      return { name, symbol, uri };
    } catch (error) {
      console.error("Error parsing metadata:", error);
      return null;
    }
  }
  function logTokenInfo(
    name: string,
    metadata: any,
    balance: bigint | number | null = null
  ) {
    console.log(`\n${name} Token Info:`);
    if (metadata) {
      console.log("  Name:", metadata.name);
      console.log("  Symbol:", metadata.symbol);
      console.log("  URI:", metadata.uri);
    }
    if (balance !== null) {
      // Форматируем bigint для читаемости
      const balanceStr =
        typeof balance === "bigint" ? balance.toString() : balance;
      console.log("  Balance:", balanceStr);
    }
  }

  // --- Добавляем переменные для User Token ---
  const tokenDecimals = 9; // Децималы мем-коина
  const nDollarDecimals = 9; // Децималы N-Dollar (убедись, что они совпадают с созданием N-Dollar)
  // // Initialize PDAs and accounts before tests
  before(async () => {
    // PDAs для N-Dollar
    [nDollarMetadataPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        METADATA_PROGRAM_ID.toBuffer(),
        nDollarMint.toBuffer(),
      ],
      METADATA_PROGRAM_ID
    );

    // PDAs и ATAs для Пула Ликвидности
    [poolPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), nDollarMint.toBuffer()],
      liquidityPoolProgram.programId
    );
    [solVaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("sol_vault"), poolPda.toBuffer()],
      liquidityPoolProgram.programId // Убедитесь, что poolPda используется как сид
    );
    poolNDollarAccount = getAssociatedTokenAddressSync(
      nDollarMint,
      poolPda,
      true
    ); // allowOwnerOffCurve = true для PDA
    userNDollarAccount = getAssociatedTokenAddressSync(
      nDollarMint,
      wallet.publicKey
    );

    // Airdrop SOL
    const userBalance = await provider.connection.getBalance(wallet.publicKey);
    if (userBalance < 20 * SOL_DECIMALS) {
      // Увеличил запас SOL на всякий случай
      await airdropSol(wallet.publicKey, 20 * SOL_DECIMALS);
    }
    console.log(
      "User SOL balance:",
      (await provider.connection.getBalance(wallet.publicKey)) / SOL_DECIMALS,
      "SOL"
    );
    console.log("N-Dollar Mint KP Pubkey:", nDollarMintKp.publicKey.toBase58());
    console.log("Pool PDA:", poolPda.toBase58());
    console.log("SOL Vault PDA:", solVaultPda.toBase58());
    console.log("Pool N-Dollar ATA:", poolNDollarAccount.toBase58());
    console.log("User N-Dollar ATA:", userNDollarAccount.toBase58());
  });

  // --- Шаги 1-5: Создание N-Dollar, инициализация и наполнение пула, свопы ---
  // --- Оставляем эти шаги как были, если они работали ---

  it("1. Creates N-Dollar token with metadata", async () => {
    // Используем NdollarProgram, если он за это отвечает
    const tx = await NdollarProgram.methods
      .createToken("N-Dollar", "ND", "https://example.com/ndollar.json") // Пример данных
      .accounts({
        mint: nDollarMint,
        metadata: nDollarMetadataPda,
        authority: wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
        tokenMetadataProgram: METADATA_PROGRAM_ID,
      })
      .signers([nDollarMintKp]) // Mint Keypair нужен как signer
      .rpc({ commitment: "confirmed" }); // Добавил confirmed для надежности

    console.log("Create N-Dollar token tx:", tx);
    await provider.connection.confirmTransaction(tx, "confirmed");

    const mintInfo = await provider.connection.getAccountInfo(nDollarMint);
    assert(mintInfo !== null, "N-Dollar mint not created");
    const metadataInfo = await provider.connection.getAccountInfo(
      nDollarMetadataPda
    );
    assert(metadataInfo !== null, "Metadata account not found");
    logTokenInfo("N-Dollar", parseMetadata(metadataInfo.data));
  });

  it("2. Initializes liquidity pool", async () => {
    // Используем NdollarProgram или liquidityPoolProgram в зависимости от того, где эта логика
    const tx = await NdollarProgram.methods // ИЛИ liquidityPoolProgram.methods.initialize(...)
      .initializeLiquidityPool() // Убедитесь, что метод и аргументы верны
      .accounts({
        mint: nDollarMint, // N-Dollar mint
        pool: poolPda,
        ndollarVault: poolNDollarAccount, // ATA пула для N-Dollar
        solVault: solVaultPda,
        authority: wallet.publicKey, // Кто инициализирует
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
        liquidityPoolProgram: liquidityPoolProgram.programId, // ID программы пула
      })
      .rpc({ commitment: "confirmed" });
    console.log("Initialize liquidity pool tx:", tx);
    await provider.connection.confirmTransaction(tx, "confirmed");

    // Проверка балансов после инициализации (если она что-то минтит)
    const ndollarVaultBalance = await getTokenBalance(
      provider,
      poolNDollarAccount
    );
    console.log("Pool N-Dollar balance after init:", ndollarVaultBalance);
    const solVaultBalance = await provider.connection.getBalance(solVaultPda);
    console.log(
      "Pool SOL balance after init:",
      solVaultBalance / SOL_DECIMALS,
      "SOL"
    );

    // Примерная проверка, если инициализация минтит 108M N-Dollar с 9 децималами
    // const expectedAmount = BigInt("108000000000000000"); // 108M * 10^9
    // assert.equal(ndollarVaultBalance, expectedAmount, `Expected ${expectedAmount} in pool vault`);
  });

  it("3. Creates user N-Dollar ATA (if needed) and adds initial liquidity", async () => {
    await sleep(1000); // Добавляем задержку перед транзакцией

    // Убедимся что ATA пользователя для N-Dollar существует
    try {
      await getAccount(provider.connection, userNDollarAccount);
      console.log("User N-Dollar ATA already exists.");
    } catch {
      console.log("Creating User N-Dollar ATA...");
      const createUserAtaTx = new anchor.web3.Transaction().add(
        createAssociatedTokenAccountInstruction(
          wallet.publicKey,
          userNDollarAccount,
          wallet.publicKey,
          nDollarMint
        )
      );
      const sig = await provider.sendAndConfirm(createUserAtaTx);
      console.log("Create User N-Dollar ATA tx:", sig);
    }

    await sleep(1000); // Добавляем задержку перед следующей транзакцией

    // Добавляем ликвидность (например, 5 SOL)
    const solAmountToAdd = new BN(5 * SOL_DECIMALS);
    const nDollarAmountToAdd = new BN(0);

    const tx = await liquidityPoolProgram.methods
      .addLiquidity(nDollarAmountToAdd, solAmountToAdd)
      .accounts({
        pool: poolPda,
        ndollarMint: nDollarMint,
        ndollarVault: poolNDollarAccount,
        solVault: solVaultPda,
        user: wallet.publicKey,
        userNdollar: userNDollarAccount,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc({ commitment: "confirmed" });
    console.log("Add liquidity tx:", tx);
    await provider.connection.confirmTransaction(tx, "confirmed");

    const poolNDollarBalance = await getTokenBalance(
      provider,
      poolNDollarAccount
    );
    console.log(
      "Pool N-Dollar balance after add liquidity:",
      poolNDollarBalance
    );
    const poolSolBalance = await provider.connection.getBalance(solVaultPda);
    console.log(
      "Pool SOL balance after add liquidity:",
      poolSolBalance / SOL_DECIMALS,
      "SOL"
    );
  });

  it("4. Swaps SOL to N-Dollar", async () => {
    await sleep(1000);

    // Проверяем балансы
    const poolSolBalance = await provider.connection.getBalance(solVaultPda);
    const poolNDollarBalance = await getTokenBalance(
      provider,
      poolNDollarAccount
    );
    console.log(
      "Pool SOL balance before swap:",
      poolSolBalance / SOL_DECIMALS,
      "SOL"
    );
    console.log(
      "Pool N-Dollar balance before swap:",
      poolNDollarBalance.toString()
    );

    const solToSwap = new BN(1 * SOL_DECIMALS);
    const userNDollarBalanceBefore = await getTokenBalance(
      provider,
      userNDollarAccount
    );
    console.log(
      "User N-Dollar balance before swap:",
      userNDollarBalanceBefore.toString()
    );

    const tx = await liquidityPoolProgram.methods
      .swapSolToNdollar(solToSwap)
      .accounts({
        pool: poolPda,
        ndollarMint: nDollarMint,
        ndollarVault: poolNDollarAccount,
        solVault: solVaultPda,
        user: wallet.publicKey,
        userNdollar: userNDollarAccount,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc({ commitment: "confirmed" });
    console.log("Swap SOL to N-Dollar tx:", tx);
    await provider.connection.confirmTransaction(tx, "confirmed");

    const userNDollarBalanceAfter = await getTokenBalance(
      provider,
      userNDollarAccount
    );
    const ndollarReceived = userNDollarBalanceAfter - userNDollarBalanceBefore;
    console.log(
      "User N-Dollar balance after swap:",
      userNDollarBalanceAfter.toString()
    );
    console.log("User received N-Dollar:", ndollarReceived.toString());
    assert(ndollarReceived > 0, "Should have received some N-Dollars");

    const finalPoolSolBalance = await provider.connection.getBalance(
      solVaultPda
    );
    console.log(
      "Pool SOL balance after swap:",
      finalPoolSolBalance / SOL_DECIMALS,
      "SOL"
    );
  });

  // Шаг 5 (Swap N-Dollar to SOL) оставляем как есть для полноты картины,
  // хотя он не строго обязателен для теста createUserToken,
  // но полезен для проверки работы пула в обе стороны.
  // it("5. Swaps N-Dollar to SOL", async () => { ... });

  // --- Шаг 6: Создание пользовательского токена с использованием N-Dollar ---
  it("6a. Creates user token and mints to distributor", async () => {
    await sleep(1000);

    userTokenMintKp = Keypair.generate();
    userTokenMint = userTokenMintKp.publicKey;
    console.log("\n6a: Creating user token...");
    console.log("User token mint pubkey:", userTokenMint.toString());

    const nDollarAmountForRent = new BN(50_000_000); // Примерное значение

    // --- Проверка и пополнение баланса N-Dollar ---
    const userNDollarBalanceBefore = await getTokenBalance(
      provider,
      userNDollarAccount
    );
    console.log(
      `User N-Dollar balance before creation: ${userNDollarBalanceBefore.toString()}`
    );
    if (userNDollarBalanceBefore < nDollarAmountForRent.toNumber()) {
      console.log("Insufficient N-Dollar balance, attempting to get more...");
      const solToSwap = new BN(2 * SOL_DECIMALS);

      const swapTx = await liquidityPoolProgram.methods
        .swapSolToNdollar(solToSwap)
        .accounts({
          pool: poolPda,
          ndollarMint: nDollarMint,
          ndollarVault: poolNDollarAccount,
          solVault: solVaultPda,
          user: wallet.publicKey,
          userNdollar: userNDollarAccount,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc({ commitment: "confirmed" });

      await provider.connection.confirmTransaction(swapTx, "confirmed");
      const newBalance = await getTokenBalance(provider, userNDollarAccount);
      console.log(`New N-Dollar balance after swap: ${newBalance.toString()}`);
      if (newBalance < nDollarAmountForRent.toNumber()) {
        throw new Error(`Still insufficient N-Dollar balance.`);
      }
    }
    assert(
      userNDollarBalanceBefore >= nDollarAmountForRent.toNumber(),
      `Insufficient N-Dollar balance.`
    );

    // --- Находим PDA и ATA, нужные для СОЗДАНИЯ ---
    const [userTokenMetadataPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        METADATA_PROGRAM_ID.toBuffer(),
        userTokenMint.toBuffer(),
      ],
      METADATA_PROGRAM_ID
    );
    const [tokenInfoPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("token_info"), userTokenMint.toBuffer()],
      tokenCreatorProgram.programId
    );

    [distributorAuthorityPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("distributor"), userTokenMint.toBuffer()],
      tokenDistributorProgram.programId // ID дистрибьютора
    );
    distributorTokenAccount = getAssociatedTokenAddressSync(
      userTokenMint,
      distributorAuthorityPda,
      true
    );

    // --- Находим PDA и ATA, которые понадобятся ПОЗЖЕ (для 6b, 7, 8, 9) ---
    [bondingCurveAuthorityPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("bonding_curve"), userTokenMint.toBuffer()],
      bondingCurveProgram.programId
    );
    [bondingCurvePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("bonding_curve"), userTokenMint.toBuffer()],
      bondingCurveProgram.programId
    );
    [nDollarTreasury] = PublicKey.findProgramAddressSync(
      [Buffer.from("n_dollar_treasury"), userTokenMint.toBuffer()],
      bondingCurveProgram.programId
    );

    userTokenAccount = getAssociatedTokenAddressSync(
      userTokenMint,
      wallet.publicKey
    );
    bondingCurveTokenAccount = getAssociatedTokenAddressSync(
      userTokenMint,
      bondingCurveAuthorityPda,
      true
    );

    // Логгирование адресов (как раньше)
    console.log("User Token Mint:", userTokenMint.toBase58());
    console.log("User Token Metadata PDA:", userTokenMetadataPda.toBase58());
    console.log("Token Info PDA:", tokenInfoPda.toBase58());
    console.log(
      "Distributor Authority PDA:",
      distributorAuthorityPda.toBase58()
    );
    console.log(
      "Bonding Curve Authority PDA:",
      bondingCurveAuthorityPda.toBase58()
    );
    console.log("Bonding Curve PDA:", bondingCurvePda.toBase58());
    console.log("N-Dollar Treasury:", nDollarTreasury.toBase58());
    console.log("User Token ATA:", userTokenAccount.toBase58());
    console.log("Distributor Token ATA:", distributorTokenAccount.toBase58());
    console.log(
      "Bonding Curve Token ATA:",
      bondingCurveTokenAccount.toBase58()
    );

    const name = "My New Token";
    const symbol = "NEWT";
    const uri = "https://example.com/newt-token.json";
    const totalSupply = new BN(100_000_000).mul(new BN(10 ** tokenDecimals)); // 1 миллиард

    try {
      // ----- ВЫЗОВ tokenCreatorProgram -----
      const txSignature = await tokenCreatorProgram.methods
        .createUserToken(name, symbol, uri, totalSupply, nDollarAmountForRent)
        .accounts({
          // Аккаунты создания токена (основные)
          mint: userTokenMint,
          metadata: userTokenMetadataPda,
          tokenInfo: tokenInfoPda,
          authority: wallet.publicKey,
          solMint: WRAPPED_SOL_MINT,

          // Аккаунты пула ликвидности (для свапа N$ -> SOL)
          liquidityPool: poolPda,
          poolNDollarAccount: poolNDollarAccount,
          poolSolAccount: solVaultPda,
          userNDollarAccount: userNDollarAccount,
          nDollarMint: nDollarMint,
          // Аккаунты, связанные с дистрибуцией (только для минта!)
          distributorAuthority: distributorAuthorityPda,
          distributorTokenAccount: distributorTokenAccount, // Сюда минтятся все токены
          // Системные программы
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
          tokenMetadataProgram: METADATA_PROGRAM_ID,
          // ID программ
          liquidityPoolProgram: liquidityPoolProgram.programId,
        })
        .preInstructions([
          anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
            units: 400_000,
          }),
        ])
        .signers([userTokenMintKp])
        .rpc({ commitment: "confirmed" });

      console.log("Create user token tx (6a):", txSignature);
      await provider.connection.confirmTransaction(txSignature, "confirmed");

      // --- ПРОВЕРКА РЕЗУЛЬТАТОВ СОЗДАНИЯ (6a) ---
      const userTokenMintInfo = await provider.connection.getAccountInfo(
        userTokenMint
      );
      assert(userTokenMintInfo, "User Token mint should exist after 6a");
      const metadataInfo = await provider.connection.getAccountInfo(
        userTokenMetadataPda
      );
      assert(metadataInfo, "User Token metadata should exist after 6a");
      const parsedMeta = parseMetadata(metadataInfo.data);

      const tokenInfoAccount =
        await tokenCreatorProgram.account.tokenInfo.fetch(tokenInfoPda);
      assert(tokenInfoAccount, "Token Info account should exist after 6a");

      // Получаем балансы ПОСЛЕ создания
      const distributorAtaBalance = await getTokenBalance(
        provider,
        distributorTokenAccount
      );
      const userAtaBalance = await getTokenBalance(provider, userTokenAccount); // Ожидаем 0
      const bondingCurveAtaBalance = await getTokenBalance(
        provider,
        bondingCurveTokenAccount
      ); // Ожидаем 0
      const userNDollarBalanceAfter = await getTokenBalance(
        provider,
        userNDollarAccount
      );
      const spentNDollars = userNDollarBalanceBefore - userNDollarBalanceAfter;

      // Логгирование для 6a
      console.log("\nToken Creation (6a) Results:");
      console.log("-----------------------------");
      logTokenInfo("User Token", parsedMeta);
      console.log(
        "Distributor Balance:",
        distributorAtaBalance.toString(),
        `(Expected: ${totalSupply.toString()})`
      );
      console.log(
        "User ATA Balance:",
        userAtaBalance.toString(),
        "(Expected: 0)"
      );
      console.log(
        "Bonding Curve ATA Balance:",
        bondingCurveAtaBalance.toString(),
        "(Expected: 0)"
      );
      console.log("Spent N-Dollars:", spentNDollars.toString());

      // --- ОСНОВНЫЕ ПРОВЕРКИ для 6a ---
      assert.equal(
        distributorAtaBalance.toString(),
        totalSupply.toString(),
        "Distributor ATA should hold the total supply after 6a"
      );
      assert.equal(
        userAtaBalance,
        BigInt(0),
        "User ATA should be empty after 6a"
      );
      assert.equal(
        bondingCurveAtaBalance,
        BigInt(0),
        "Bonding Curve ATA should be empty after 6a"
      );

      // Проверка потраченных N-Dollar (остается как была)
      const maxAllowedSpent =
        (BigInt(nDollarAmountForRent.toString()) * BigInt(110)) / BigInt(100);
      assert(
        spentNDollars > 0 && spentNDollars <= maxAllowedSpent,
        `N-Dollar spent amount is outside allowed range`
      );

      console.log(
        "Token creation (6a) successful. Tokens minted to distributor."
      );
    } catch (error) {
      console.error("Error creating user token (6a):", error);
      if (error.logs) {
        console.error("Transaction Logs:", error.logs);
      }
      throw error;
    }
  });

  // --- Шаг 6b: Распределение токенов ---
  it("6b. Distributes tokens from distributor account", async () => {
    await sleep(2000); // Даем время ноде синхронизироваться
    console.log("\n6b: Distributing tokens...");

    // Убедимся, что переменные из 6a доступны
    assert(userTokenMint, "userTokenMint is not defined from step 6a");
    assert(distributorAuthorityPda, "distributorAuthorityPda not defined");
    assert(distributorTokenAccount, "distributorTokenAccount not defined");
    assert(userTokenAccount, "userTokenAccount not defined");
    assert(bondingCurveAuthorityPda, "bondingCurveAuthorityPda not defined");
    assert(bondingCurveTokenAccount, "bondingCurveTokenAccount not defined");

    // Переопределяем tokenInfoPda, так как он мог потеряться между тестами
    const [tokenInfoPdaLocal] = PublicKey.findProgramAddressSync(
      [Buffer.from("token_info"), userTokenMint.toBuffer()],
      tokenCreatorProgram.programId
    );
    tokenInfoPda = tokenInfoPdaLocal;
    console.log("Re-derived tokenInfoPda:", tokenInfoPda.toString());

    // Получаем total supply ИЗ TokenInfo, созданного в 6a
    const tokenInfo = await tokenCreatorProgram.account.tokenInfo.fetch(
      tokenInfoPda
    );
    const totalSupplyBigInt = BigInt(tokenInfo.totalSupply.toString());
    console.log(
      "Total supply fetched from TokenInfo:",
      totalSupplyBigInt.toString()
    );

    // Получаем балансы ДО распределения
    const distributorBalanceBefore = await getTokenBalance(
      provider,
      distributorTokenAccount
    );
    const userBalanceBefore = await getTokenBalance(provider, userTokenAccount);
    const curveBalanceBefore = await getTokenBalance(
      provider,
      bondingCurveTokenAccount
    );

    console.log("Balances BEFORE distribution (6b):");
    console.log("  Distributor:", distributorBalanceBefore.toString());
    console.log("  User:", userBalanceBefore.toString());
    console.log("  Bonding Curve:", curveBalanceBefore.toString());

    // Проверяем, что дистрибьютор действительно содержит токены
    assert.equal(
      distributorBalanceBefore,
      totalSupplyBigInt,
      "Distributor balance mismatch before distribution (6b)"
    );

    try {
      // ----- ВЫЗОВ tokenDistributorProgram -----
      const txSignature = await tokenDistributorProgram.methods
        .distributeTokens() // total_supply больше не передается
        .accounts({
          mint: userTokenMint,
          distributorAuthority: distributorAuthorityPda,
          distributorTokenAccount: distributorTokenAccount, // Откуда
          userAuthority: wallet.publicKey, // Кому 70% + Payer
          userTokenAccount: userTokenAccount, // Куда 70%
          bondingCurveAuthority: bondingCurveAuthorityPda, // Авторитет для 30% ATA
          bondingCurveTokenAccount: bondingCurveTokenAccount, // Куда 30%
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        })
        .rpc({ commitment: "confirmed" });

      console.log("Distribute tokens tx (6b):", txSignature);
      await provider.connection.confirmTransaction(txSignature, "confirmed");

      // --- ПРОВЕРКА РЕЗУЛЬТАТОВ РАСПРЕДЕЛЕНИЯ (6b) ---
      const distributorAtaBalanceAfter = await getTokenBalance(
        provider,
        distributorTokenAccount
      );
      const userAtaBalanceAfter = await getTokenBalance(
        provider,
        userTokenAccount
      );
      const bondingCurveAtaBalanceAfter = await getTokenBalance(
        provider,
        bondingCurveTokenAccount
      );

      const expectedUserAmount = (totalSupplyBigInt * BigInt(70)) / BigInt(100);
      const expectedBondingCurveAmount = totalSupplyBigInt - expectedUserAmount; // Остаток

      console.log("\nToken Distribution (6b) Results:");
      console.log("--------------------------------");
      console.log(
        "Distributor Balance:",
        distributorAtaBalanceAfter.toString(),
        "(Expected: 0)"
      );
      console.log(
        "User ATA Balance:",
        userAtaBalanceAfter.toString(),
        `(Expected: ${expectedUserAmount.toString()})`
      );
      console.log(
        "Bonding Curve ATA Balance:",
        bondingCurveAtaBalanceAfter.toString(),
        `(Expected: ${expectedBondingCurveAmount.toString()})`
      );

      // --- ОСНОВНЫЕ ПРОВЕРКИ для 6b ---
      assert.equal(
        distributorAtaBalanceAfter,
        BigInt(0),
        "Distributor ATA should be empty after 6b"
      );
      assert.equal(
        userAtaBalanceAfter.toString(),
        expectedUserAmount.toString(),
        "User ATA balance mismatch after 6b (70%)"
      );
      assert.equal(
        bondingCurveAtaBalanceAfter.toString(),
        expectedBondingCurveAmount.toString(),
        "Bonding Curve ATA balance mismatch after 6b (30%)"
      );

      console.log("Token distribution (6b) successful.");
    } catch (error) {
      console.error("Error distributing tokens (6b):", error);
      if (error.logs) {
        console.error("Transaction Logs:", error.logs);
      }
      throw error;
    }
  });

  // --- Шаг 7: Инициализация Бондинг Кривой ---
  it("7. Initializes the bonding curve", async () => {
    await sleep(1000);
    const userBalance = await provider.connection.getBalance(wallet.publicKey);
    if (userBalance < 20 * SOL_DECIMALS) {
      // Увеличил запас SOL на всякий случай
      await airdropSol(wallet.publicKey, 20 * SOL_DECIMALS);
    }

    console.log("\nInitializing bonding curve for:", userTokenMint.toBase58());

    // Проверяем, что нужные переменные userTokenMint, nDollarMint, etc. доступны из предыдущего шага
    assert(userTokenMint, "userTokenMint not set");
    assert(bondingCurvePda, "bondingCurvePda not set");
    assert(bondingCurveTokenAccount, "bondingCurveTokenAccount not set");
    assert(nDollarTreasury, "nDollarTreasury not set");
    assert(bondingCurveAuthorityPda, "bondingCurveAuthorityPda not set");

    const actualBalanceBeforeInit = await getTokenBalance(
      provider,
      bondingCurveTokenAccount
    );
    console.log(
      `Actual balance in bondingCurveTokenAccount before init: ${actualBalanceBeforeInit.toString()}`
    );
    const expectedBalance = BigInt(
      new BN(30_000_000).mul(new BN(10 ** tokenDecimals)).toString()
    );
    console.log(
      `Expected balance for bonding curve: ${expectedBalance.toString()}`
    );
    // Можно добавить assert здесь для ранней проверки
    assert.equal(
      actualBalanceBeforeInit,
      expectedBalance,
      "Balance mismatch BEFORE calling initializeCurve"
    );

    try {
      const txSignature = await bondingCurveProgram.methods
        .initializeCurve()
        .accounts({
          bondingCurve: bondingCurvePda, // PDA состояния (создается)
          mint: userTokenMint,
          nDollarMint: nDollarMint,
          bondingCurveTokenAccount: bondingCurveTokenAccount, // ATA с 30% токенов (уже существует)
          nDollarTreasury: nDollarTreasury, // ATA казны (создается)
          bondingCurveAuthority: bondingCurveAuthorityPda, // PDA авторитета (для казны)
          authority: wallet.publicKey, // Payer
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .rpc({ commitment: "confirmed" });

      // console.log("Initialize bonding curve tx:", txSignature);
      await provider.connection.confirmTransaction(txSignature, "confirmed");

      // console.log("Workspace Keys:", Object.keys(anchor.workspace));
      // console.log(
      //   "Is BondingCurve program defined?",
      //   !!anchor.workspace.BondingCurve
      // );
      // console.log(
      //   "Bonding Curve Program Object:",
      //   anchor.workspace.BondingCurve
      // );

      const curveAccountInfo = await provider.connection.getAccountInfo(
        bondingCurvePda
      );
      assert(
        curveAccountInfo,
        "Bonding curve state account not found after tx"
      );
      console.log("Curve PDA account info fetched successfully via web3.js");

      console.log(
        "Attempting to fetch BondingCurve account data via Anchor..."
      );
      const curveAccount = await bondingCurveProgram.account.bondingCurve.fetch(
        bondingCurvePda
      );

      assert(curveAccount.isInitialized, "Bonding curve not initialized");
      assert.equal(
        curveAccount.mint.toBase58(),
        userTokenMint.toBase58(),
        "Mint mismatch"
      );
      assert.equal(
        curveAccount.nDollarMint.toBase58(),
        nDollarMint.toBase58(),
        "N-Dollar mint mismatch"
      );

      console.log("Bonding curve state initialized:");
      console.log("  Slope Numerator:", curveAccount.slopeNumerator.toString());
      console.log(
        "  Slope Denominator:",
        curveAccount.slopeDenominator.toString()
      );
      console.log(
        "  Intercept Scaled:",
        curveAccount.interceptScaled.toString()
      );
      console.log(
        "  Initial Supply:",
        curveAccount.initialBondingSupply.toString()
      );

      // Проверяем, что казна создана и пуста
      const treasuryBalance = await getTokenBalance(provider, nDollarTreasury);
      assert.equal(
        treasuryBalance,
        BigInt(0),
        "N-Dollar treasury should be empty initially"
      );
      console.log("N-Dollar treasury created and empty.");
    } catch (error) {
      console.error("Error initializing bonding curve:", error);
      if (error.logs) {
        console.error("Logs:", error.logs);
      }
      throw error;
    }
  });

  // Добавляем после других utility functions
  function calculateTokenValue(amount: bigint, curveAccount: any): bigint {
    const PRECISION_FACTOR = BigInt(1_000_000_000_000); // 10^12 как в контракте
    const tokenDecimalFactor = BigInt(10 ** curveAccount.tokenDecimals);

    // Переводим amount к базовым единицам (без децималов)
    const amountBase = amount / tokenDecimalFactor;

    const slopeNum = BigInt(curveAccount.slopeNumerator.toString());
    const slopeDenom = BigInt(curveAccount.slopeDenominator.toString());
    const intercept = BigInt(curveAccount.interceptScaled.toString());

    // Формула из контракта:
    // Для покупки: cost = (m/2 * ((x+dx)^2 - x^2) + c*dx)
    // Для текущей стоимости всех токенов: value = (m/2 * x^2 + c*x)
    const term1 =
      (slopeNum * amountBase * amountBase) / (slopeDenom * BigInt(2));
    const term2 = (intercept * amountBase) / PRECISION_FACTOR;

    // Переводим результат в N-Dollar lamports
    const nDollarDecimalFactor = BigInt(10 ** curveAccount.nDollarDecimals);
    const value = ((term1 + term2) * nDollarDecimalFactor) / PRECISION_FACTOR;

    return value;
  }

  // --- Шаг 8: Покупка Токенов с Кривой ---
  it("8. Buys tokens from the bonding curve", async () => {
    await sleep(1000);
    console.log("\nBuying tokens from bonding curve...");

    // Проверяем, что все необходимые переменные определены
    assert(userTokenMint, "userTokenMint not defined");
    assert(bondingCurvePda, "bondingCurvePda not defined");
    assert(bondingCurveTokenAccount, "bondingCurveTokenAccount not defined");
    assert(nDollarTreasury, "nDollarTreasury not defined");
    assert(bondingCurveAuthorityPda, "bondingCurveAuthorityPda not defined");
    assert(userTokenAccount, "userTokenAccount not defined");

    // Проверяем, что бондинг кривая инициализирована
    const curveAccount = await bondingCurveProgram.account.bondingCurve.fetch(
      bondingCurvePda
    );
    assert(curveAccount.isInitialized, "Bonding curve not initialized");

    // Получаем начальный баланс и стоимость
    const userTokenBalanceBefore = await getTokenBalance(
      provider,
      userTokenAccount
    );
    console.log("\nТокены пользователя ДО покупки:");
    console.log(`Баланс: ${userTokenBalanceBefore.toString()}`);
    console.log(
      `Стоимость в N-Dollar: ${calculateTokenValue(
        userTokenBalanceBefore,
        curveAccount
      ).toString()}`
    );

    // Сколько токенов покупаем (например, 1000 с децималами)
    const amountToBuy = new BN(1000).mul(new BN(10 ** tokenDecimals));
    const amountToBuyBigInt = BigInt(amountToBuy.toString());

    // Получаем балансы ДО покупки
    const userNDollarBalanceBefore = await getTokenBalance(
      provider,
      userNDollarAccount
    );
    const curveTokenBalanceBefore = await getTokenBalance(
      provider,
      bondingCurveTokenAccount
    );

    // Проверяем баланс казны, если она существует
    let treasuryBalanceBefore = BigInt(0);
    try {
      treasuryBalanceBefore = await getTokenBalance(provider, nDollarTreasury);
    } catch (error) {
      console.log("N-Dollar treasury not created yet, assuming 0 balance");
    }

    console.log(`Attempting to buy ${amountToBuy.toString()} tokens`);
    console.log("Balances BEFORE buy:");
    console.log("  User Tokens:", userTokenBalanceBefore.toString());
    console.log("  Curve Tokens:", curveTokenBalanceBefore.toString());
    console.log("  User N-Dollars:", userNDollarBalanceBefore.toString());
    console.log("  Treasury N-Dollars:", treasuryBalanceBefore.toString());

    assert(userNDollarBalanceBefore > 0, "User has no N-Dollars to buy");

    try {
      const txSignature = await bondingCurveProgram.methods
        .buy(amountToBuy)
        .accounts({
          bondingCurve: bondingCurvePda,
          mint: userTokenMint,
          nDollarMint: nDollarMint,
          bondingCurveTokenAccount: bondingCurveTokenAccount,
          nDollarTreasury: nDollarTreasury,
          userTokenAccount: userTokenAccount,
          userNDollarAccount: userNDollarAccount,
          bondingCurveAuthority: bondingCurveAuthorityPda,
          userAuthority: wallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc({ commitment: "confirmed" });

      console.log("Buy transaction tx:", txSignature);
      await provider.connection.confirmTransaction(txSignature, "confirmed");

      // Получаем балансы ПОСЛЕ покупки
      const userTokenBalanceAfter = await getTokenBalance(
        provider,
        userTokenAccount
      );
      const curveTokenBalanceAfter = await getTokenBalance(
        provider,
        bondingCurveTokenAccount
      );
      const userNDollarBalanceAfter = await getTokenBalance(
        provider,
        userNDollarAccount
      );

      // Проверяем баланс казны после покупки
      let treasuryBalanceAfter = BigInt(0);
      try {
        treasuryBalanceAfter = await getTokenBalance(provider, nDollarTreasury);
      } catch (error) {
        console.log("N-Dollar treasury not created yet, assuming 0 balance");
      }

      console.log("\nBalances AFTER buy:");
      console.log("  User Tokens:", userTokenBalanceAfter.toString());
      console.log("  Curve Tokens:", curveTokenBalanceAfter.toString());
      console.log("  User N-Dollars:", userNDollarBalanceAfter.toString());
      console.log("  Treasury N-Dollars:", treasuryBalanceAfter.toString());

      const tokensReceived = userTokenBalanceAfter - userTokenBalanceBefore;
      const tokensSent = curveTokenBalanceBefore - curveTokenBalanceAfter;
      const nDollarsSpent = userNDollarBalanceBefore - userNDollarBalanceAfter;
      const nDollarsReceivedTreasury =
        treasuryBalanceAfter - treasuryBalanceBefore;

      // Проверки
      assert.equal(
        tokensReceived,
        amountToBuyBigInt,
        "User did not receive correct amount of tokens"
      );
      assert.equal(
        tokensSent,
        amountToBuyBigInt,
        "Curve did not send correct amount of tokens"
      );
      assert(nDollarsSpent > 0, "User did not spend any N-Dollars");
      assert.equal(
        nDollarsReceivedTreasury,
        nDollarsSpent,
        "Treasury did not receive the spent N-Dollars"
      );

      console.log(`User received ${tokensReceived} tokens.`);
      console.log(`User spent ${nDollarsSpent} N-Dollar lamports.`);

      // После покупки
      const userTokenBalanceAfterPost = await getTokenBalance(
        provider,
        userTokenAccount
      );
      console.log("\nТокены пользователя ПОСЛЕ покупки:");
      console.log(`Баланс: ${userTokenBalanceAfterPost.toString()}`);
      console.log(
        `Стоимость в N-Dollar: ${calculateTokenValue(
          userTokenBalanceAfterPost,
          curveAccount
        ).toString()}`
      );
    } catch (error) {
      console.error("Error buying from bonding curve:", error);
      if (error.logs) {
        console.error("Logs:", error.logs);
      }
      throw error;
    }
  });

  // --- Шаг 9: Продажа Токенов на Кривую ---
  it("9. Sells tokens to the bonding curve", async () => {
    await sleep(1000);
    console.log("\nSelling tokens to bonding curve...");

    // Проверяем, что все необходимые переменные определены
    assert(userTokenMint, "userTokenMint not defined");
    assert(bondingCurvePda, "bondingCurvePda not defined");
    assert(bondingCurveTokenAccount, "bondingCurveTokenAccount not defined");
    assert(nDollarTreasury, "nDollarTreasury not defined");
    assert(bondingCurveAuthorityPda, "bondingCurveAuthorityPda not defined");
    assert(userTokenAccount, "userTokenAccount not defined");

    // Проверяем, что бондинг кривая инициализирована
    const curveAccount = await bondingCurveProgram.account.bondingCurve.fetch(
      bondingCurvePda
    );
    assert(curveAccount.isInitialized, "Bonding curve not initialized");

    // Получаем начальный баланс и стоимость
    const userTokenBalanceBeforeSell = await getTokenBalance(
      provider,
      userTokenAccount
    );
    console.log("\nТокены пользователя ДО продажи:");
    console.log(`Баланс: ${userTokenBalanceBeforeSell.toString()}`);
    console.log(
      `Стоимость в N-Dollar: ${calculateTokenValue(
        userTokenBalanceBeforeSell,
        curveAccount
      ).toString()}`
    );

    // Сколько токенов продаем (например, половину купленных)
    const amountToSell = new BN(10000000); // Продаем половину
    const amountToSellBigInt = BigInt(amountToSell.toString());

    assert(amountToSellBigInt > 0, "Cannot sell zero tokens");

    // Получаем балансы ДО продажи
    const curveTokenBalanceBefore = await getTokenBalance(
      provider,
      bondingCurveTokenAccount
    );
    const userNDollarBalanceBefore = await getTokenBalance(
      provider,
      userNDollarAccount
    );

    // Проверяем баланс казны, если она существует
    let treasuryBalanceBefore = BigInt(0);
    try {
      treasuryBalanceBefore = await getTokenBalance(provider, nDollarTreasury);
    } catch (error) {
      console.log("N-Dollar treasury not created yet, assuming 0 balance");
    }

    console.log(`Attempting to sell ${amountToSell.toString()} tokens`);
    console.log("Balances BEFORE sell:");
    console.log("  User Tokens:", userTokenBalanceBeforeSell.toString());
    console.log("  Curve Tokens:", curveTokenBalanceBefore.toString());
    console.log("  User N-Dollars:", userNDollarBalanceBefore.toString());
    console.log("  Treasury N-Dollars:", treasuryBalanceBefore.toString());

    assert(treasuryBalanceBefore > 0, "Treasury has no N-Dollars to pay");

    try {
      const txSignature = await bondingCurveProgram.methods
        .sell(amountToSell)
        .accounts({
          bondingCurve: bondingCurvePda,
          mint: userTokenMint,
          nDollarMint: nDollarMint,
          bondingCurveTokenAccount: bondingCurveTokenAccount,
          nDollarTreasury: nDollarTreasury,
          userTokenAccount: userTokenAccount,
          userNDollarAccount: userNDollarAccount,
          bondingCurveAuthority: bondingCurveAuthorityPda,
          userAuthority: wallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc({ commitment: "confirmed" });

      console.log("Sell transaction tx:", txSignature);
      await provider.connection.confirmTransaction(txSignature, "confirmed");

      // Получаем балансы ПОСЛЕ продажи
      const userTokenBalanceAfter = await getTokenBalance(
        provider,
        userTokenAccount
      );
      const curveTokenBalanceAfter = await getTokenBalance(
        provider,
        bondingCurveTokenAccount
      );
      const userNDollarBalanceAfter = await getTokenBalance(
        provider,
        userNDollarAccount
      );

      // Проверяем баланс казны после продажи
      let treasuryBalanceAfter = BigInt(0);
      try {
        treasuryBalanceAfter = await getTokenBalance(provider, nDollarTreasury);
      } catch (error) {
        console.log("N-Dollar treasury not created yet, assuming 0 balance");
      }

      console.log("\nBalances AFTER sell:");
      console.log("  User Tokens:", userTokenBalanceAfter.toString());
      console.log("  Curve Tokens:", curveTokenBalanceAfter.toString());
      console.log("  User N-Dollars:", userNDollarBalanceAfter.toString());
      console.log("  Treasury N-Dollars:", treasuryBalanceAfter.toString());

      const tokensSold = userTokenBalanceBeforeSell - userTokenBalanceAfter;
      const tokensReceivedCurve =
        curveTokenBalanceAfter - curveTokenBalanceBefore;
      const nDollarsReceived =
        userNDollarBalanceAfter - userNDollarBalanceBefore;
      const nDollarsSpentTreasury =
        treasuryBalanceBefore - treasuryBalanceAfter;

      // Проверки
      assert.equal(
        tokensSold,
        amountToSellBigInt,
        "User did not sell correct amount of tokens"
      );
      assert.equal(
        tokensReceivedCurve,
        amountToSellBigInt,
        "Curve did not receive correct amount of tokens"
      );
      assert(nDollarsReceived > 0, "User did not receive any N-Dollars");
      assert.equal(
        nDollarsSpentTreasury,
        nDollarsReceived,
        "Treasury did not spend the received N-Dollars"
      );

      console.log(`User sold ${tokensSold} tokens.`);
      console.log(`User received ${nDollarsReceived} N-Dollar lamports.`);

      // После продажи
      const userTokenBalanceAfterPost = await getTokenBalance(
        provider,
        userTokenAccount
      );
      console.log("\nТокены пользователя ПОСЛЕ продажи:");
      console.log(`Баланс: ${userTokenBalanceAfterPost.toString()}`);
      console.log(
        `Стоимость в N-Dollar: ${calculateTokenValue(
          userTokenBalanceAfterPost,
          curveAccount
        ).toString()}`
      );
    } catch (error) {
      console.error("Error selling to bonding curve:", error);
      if (error.logs) {
        console.error("Logs:", error.logs);
      }
      throw error;
    }
  });
});
