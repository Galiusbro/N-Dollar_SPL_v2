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

    // Добавляем ликвидность (например, 10 SOL)
    const solAmountToAdd = new BN(10 * SOL_DECIMALS);
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

    // --- ЛОГИРОВАНИЕ ВСЕХ КЛЮЧЕВЫХ ПЕРЕМЕННЫХ И PDA ---
    console.log("userTokenMint", userTokenMint?.toBase58?.() ?? userTokenMint);
    // PDA для metadata
    const [userTokenMetadataPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        METADATA_PROGRAM_ID.toBuffer(),
        userTokenMint.toBuffer(),
      ],
      METADATA_PROGRAM_ID
    );
    console.log(
      "userTokenMetadataPda",
      userTokenMetadataPda?.toBase58?.() ?? userTokenMetadataPda
    );
    // PDA для token_info
    const [tokenInfoPdaLocal] = PublicKey.findProgramAddressSync(
      [Buffer.from("token_info"), userTokenMint.toBuffer()],
      tokenCreatorProgram.programId
    );
    tokenInfoPda = tokenInfoPdaLocal;
    console.log("tokenInfoPda", tokenInfoPda?.toBase58?.() ?? tokenInfoPda);
    // PDA для distributor
    [distributorAuthorityPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("distributor"), userTokenMint.toBuffer()],
      tokenDistributorProgram.programId
    );
    console.log(
      "distributorAuthorityPda",
      distributorAuthorityPda?.toBase58?.() ?? distributorAuthorityPda
    );
    // PDA для bonding_curve
    [bondingCurveAuthorityPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("bonding_curve"), userTokenMint.toBuffer()],
      bondingCurveProgram.programId
    );
    [bondingCurvePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("bonding_curve"), userTokenMint.toBuffer()],
      bondingCurveProgram.programId
    );
    console.log(
      "bondingCurveAuthorityPda",
      bondingCurveAuthorityPda?.toBase58?.() ?? bondingCurveAuthorityPda
    );
    console.log(
      "bondingCurvePda",
      bondingCurvePda?.toBase58?.() ?? bondingCurvePda
    );
    // PDA для n_dollar_treasury
    nDollarTreasury = getAssociatedTokenAddressSync(
      nDollarMint,
      bondingCurvePda,
      true // allowOwnerOffCurve = true для PDA
    );
    console.log(
      "nDollarTreasury (ATA for nDollarMint & bondingCurvePda):",
      nDollarTreasury?.toBase58?.() ?? nDollarTreasury
    );
    // ATA для пользователя
    userTokenAccount = getAssociatedTokenAddressSync(
      userTokenMint,
      wallet.publicKey
    );
    console.log(
      "userTokenAccount",
      userTokenAccount?.toBase58?.() ?? userTokenAccount
    );
    // ATA для дистрибьютора
    distributorTokenAccount = getAssociatedTokenAddressSync(
      userTokenMint,
      distributorAuthorityPda,
      true
    );
    console.log(
      "distributorTokenAccount",
      distributorTokenAccount?.toBase58?.() ?? distributorTokenAccount
    );
    // ATA для bonding_curve
    bondingCurveTokenAccount = getAssociatedTokenAddressSync(
      userTokenMint,
      bondingCurveAuthorityPda,
      true
    );
    console.log(
      "bondingCurveTokenAccount",
      bondingCurveTokenAccount?.toBase58?.() ?? bondingCurveTokenAccount
    );
    // --- КОНЕЦ ЛОГИРОВАНИЯ ---

    const name = "My New Token";
    const symbol = "NEWT";
    const uri = "https://example.com/newt-token.json";
    const totalSupply = new BN(100_000_000).mul(new BN(10 ** tokenDecimals)); // 1 миллиард

    try {
      // ----- ВЫЗОВ tokenCreatorProgram -----
      const txSignature = await tokenCreatorProgram.methods
        .createUserToken(name, symbol, uri, totalSupply)
        .accounts({
          mint: userTokenMint,
          metadata: userTokenMetadataPda,
          tokenInfo: tokenInfoPda,
          authority: wallet.publicKey,
          rentPayer: wallet.publicKey,
          distributorAuthority: distributorAuthorityPda,
          distributorTokenAccount: distributorTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
          tokenMetadataProgram: METADATA_PROGRAM_ID,
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

      // --- ПРОВЕРКА: Существует ли tokenInfoAccount? ---
      const tokenInfoAccountInfo = await provider.connection.getAccountInfo(
        tokenInfoPda
      );
      console.log("tokenInfoAccountInfo", tokenInfoAccountInfo);

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

      // @ts-ignore
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
      // const userNDollarBalanceAfter = await getTokenBalance(
      //   provider,
      //   userNDollarAccount
      // );
      // const spentNDollars = userNDollarBalanceBefore - userNDollarBalanceAfter;

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
      // console.log("Spent N-Dollars:", spentNDollars.toString());

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
      // const maxAllowedSpent =
      //   (BigInt(nDollarAmountForRent.toString()) * BigInt(110)) / BigInt(100);
      // assert(
      //   spentNDollars > 0 && spentNDollars <= maxAllowedSpent,
      //   `N-Dollar spent amount is outside allowed range`
      // );

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
    assert(userTokenAccount, "userTokenAccount not defined"); // This is ATA for wallet.publicKey (user_authority)
    assert(bondingCurveAuthorityPda, "bondingCurveAuthorityPda not defined");
    assert(bondingCurveTokenAccount, "bondingCurveTokenAccount not defined");

    // ---- Новые переменные для distributeTokens ----
    const REFERRAL_PROGRAM_ID = new PublicKey(
      "DMQh8Evpe3y4DzAWxx1rhLuGpnZGDvFSPLJvD9deQQfX"
    );
    const aiAgentAuthorityKp = Keypair.generate();
    const aiAgentAuthority = aiAgentAuthorityKp.publicKey;
    console.log("AI Agent Authority Pubkey:", aiAgentAuthority.toBase58());

    const [referralTreasuryAuthorityPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("referral_treasury"), userTokenMint.toBuffer()],
      REFERRAL_PROGRAM_ID // ID реферальной программы
    );
    const referralTreasuryTokenAccount = getAssociatedTokenAddressSync(
      userTokenMint,
      referralTreasuryAuthorityPda,
      true // allowOwnerOffCurve = true для PDA
    );
    const aiAgentTokenAccount = getAssociatedTokenAddressSync(
      userTokenMint,
      aiAgentAuthority
    );

    console.log(
      "Referral Treasury Authority PDA:",
      referralTreasuryAuthorityPda.toBase58()
    );
    console.log(
      "Referral Treasury Token ATA:",
      referralTreasuryTokenAccount.toBase58()
    );
    console.log("AI Agent Token ATA:", aiAgentTokenAccount.toBase58());
    // --------------------------------------------

    // Переопределяем tokenInfoPda, так как он мог потеряться между тестами
    // или если тест 6a не прошел и не присвоил глобальную переменную
    const [tokenInfoPdaLocalFor6b] = PublicKey.findProgramAddressSync(
      [Buffer.from("token_info"), userTokenMint.toBuffer()],
      tokenCreatorProgram.programId
    );
    tokenInfoPda = tokenInfoPdaLocalFor6b; // Присваиваем глобальной переменной
    console.log("Re-derived tokenInfoPda for 6b:", tokenInfoPda.toString());

    // Получаем total supply ИЗ TokenInfo, созданного в 6a
    // @ts-ignore
    const tokenInfo = await tokenCreatorProgram.account.tokenInfo.fetch(
      tokenInfoPda // Используем глобальную tokenInfoPda
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
    const userBalanceBefore = await getTokenBalance(provider, userTokenAccount); // Баланс userTokenAccount (для wallet.publicKey)
    const curveBalanceBefore = await getTokenBalance(
      provider,
      bondingCurveTokenAccount
    );
    const aiAgentBalanceBefore = await getTokenBalance(
      provider,
      aiAgentTokenAccount
    );
    const referralTreasuryBalanceBefore = await getTokenBalance(
      provider,
      referralTreasuryTokenAccount
    );

    console.log("Balances BEFORE distribution (6b):");
    console.log("  Distributor:", distributorBalanceBefore.toString());
    console.log("  User (wallet):", userBalanceBefore.toString());
    console.log("  Bonding Curve:", curveBalanceBefore.toString());
    console.log("  AI Agent:", aiAgentBalanceBefore.toString());
    console.log(
      "  Referral Treasury:",
      referralTreasuryBalanceBefore.toString()
    );

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
          distributorTokenAccount: distributorTokenAccount,
          userAuthority: wallet.publicKey, // user_authority (для user_token_account)
          rentPayer: wallet.publicKey, // rent_payer
          userTokenAccount: userTokenAccount, // user_token_account (для wallet.publicKey, не основной получатель)
          bondingCurveAuthority: bondingCurveAuthorityPda,
          bondingCurveTokenAccount: bondingCurveTokenAccount, // 40% сюда
          referralTreasuryAuthority: referralTreasuryAuthorityPda,
          referralTreasuryTokenAccount: referralTreasuryTokenAccount, // 10% сюда
          aiAgentAuthority: aiAgentAuthority, // Адрес AI агента
          aiAgentTokenAccount: aiAgentTokenAccount, // 50% сюда
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          referralProgram: REFERRAL_PROGRAM_ID, // ID реферальной программы
        })
        .preInstructions([
          anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
            units: 600_000, // Увеличиваем лимит CU
          }),
        ])
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
        userTokenAccount // Баланс userTokenAccount (для wallet.publicKey)
      );
      const bondingCurveAtaBalanceAfter = await getTokenBalance(
        provider,
        bondingCurveTokenAccount
      );
      const aiAgentTokenAccountBalanceAfter = await getTokenBalance(
        provider,
        aiAgentTokenAccount
      );
      const referralTreasuryTokenAccountBalanceAfter = await getTokenBalance(
        provider,
        referralTreasuryTokenAccount
      );

      const expectedReferralAmount =
        (totalSupplyBigInt * BigInt(10)) / BigInt(100);
      const expectedBondingCurveAmount =
        (totalSupplyBigInt * BigInt(40)) / BigInt(100);
      const expectedAiAgentAmount =
        totalSupplyBigInt - expectedReferralAmount - expectedBondingCurveAmount; // 50%

      console.log("\nToken Distribution (6b) Results:");
      console.log("--------------------------------");
      console.log(
        "Distributor Balance:",
        distributorAtaBalanceAfter.toString(),
        "(Expected: 0)"
      );
      console.log(
        "User (wallet.publicKey) ATA Balance:", // Этот аккаунт не должен получать токены по новой логике
        userAtaBalanceAfter.toString(),
        `(Expected: 0 or initial if pre-existing)`
      );
      console.log(
        "Bonding Curve ATA Balance:",
        bondingCurveAtaBalanceAfter.toString(),
        `(Expected: ${expectedBondingCurveAmount.toString()})`
      );
      console.log(
        "AI Agent ATA Balance:",
        aiAgentTokenAccountBalanceAfter.toString(),
        `(Expected: ${expectedAiAgentAmount.toString()})`
      );
      console.log(
        "Referral Treasury ATA Balance:",
        referralTreasuryTokenAccountBalanceAfter.toString(),
        `(Expected: ${expectedReferralAmount.toString()})`
      );

      // --- ОСНОВНЫЕ ПРОВЕРКИ для 6b ---
      assert.equal(
        distributorAtaBalanceAfter,
        BigInt(0),
        "Distributor ATA should be empty after 6b"
      );
      // userAtaBalanceAfter should ideally be userBalanceBefore, if it wasn't 0.
      // For simplicity, if it was 0, it should remain 0.
      assert.equal(
        userAtaBalanceAfter.toString(),
        userBalanceBefore.toString(), // Expecting it to be unchanged by this specific distribution logic
        "User (wallet) ATA balance should remain unchanged by this distribution"
      );
      assert.equal(
        bondingCurveAtaBalanceAfter.toString(),
        expectedBondingCurveAmount.toString(),
        "Bonding Curve ATA balance mismatch after 6b (40%)"
      );
      assert.equal(
        aiAgentTokenAccountBalanceAfter.toString(),
        expectedAiAgentAmount.toString(),
        "AI Agent ATA balance mismatch after 6b (50%)"
      );
      assert.equal(
        referralTreasuryTokenAccountBalanceAfter.toString(),
        expectedReferralAmount.toString(),
        "Referral Treasury ATA balance mismatch after 6b (10%)"
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
      await airdropSol(wallet.publicKey, 20 * SOL_DECIMALS);
    }

    console.log("\nInitializing bonding curve for:", userTokenMint.toBase58());

    // Проверяем, что нужные переменные userTokenMint, nDollarMint, etc. доступны из предыдущего шага
    assert(userTokenMint, "userTokenMint not set");
    assert(bondingCurvePda, "bondingCurvePda not set");
    assert(nDollarTreasury, "nDollarTreasury not set");
    assert(bondingCurveAuthorityPda, "bondingCurveAuthorityPda not set");

    // --- Проверяем/создаём ATA для bondingCurvePda ---
    const bondingCurveTokenAccount = getAssociatedTokenAddressSync(
      userTokenMint,
      bondingCurvePda,
      true
    );
    let needCreateBondingCurveATA = false;
    try {
      const ataInfo = await getAccount(
        provider.connection,
        bondingCurveTokenAccount
      );
      console.log("bondingCurveTokenAccount owner:", ataInfo.owner.toBase58());
      if (ataInfo.owner.toBase58() !== bondingCurvePda.toBase58()) {
        throw new Error("bondingCurveTokenAccount owner mismatch!");
      }
    } catch (e) {
      console.log(
        "bondingCurveTokenAccount does not exist or wrong owner, will create:",
        e.message
      );
      needCreateBondingCurveATA = true;
    }
    if (needCreateBondingCurveATA) {
      const createATAIx = createAssociatedTokenAccountInstruction(
        wallet.publicKey, // payer
        bondingCurveTokenAccount,
        bondingCurvePda, // owner
        userTokenMint
      );
      const tx = new anchor.web3.Transaction().add(createATAIx);
      const sig = await provider.sendAndConfirm(tx);
      console.log("Created bondingCurveTokenAccount ATA:", sig);
    }

    // --- Дальше всё как было ---
    const actualBalanceBeforeInit = await getTokenBalance(
      provider,
      bondingCurveTokenAccount
    );
    console.log(
      `Actual balance in bondingCurveTokenAccount before init: ${actualBalanceBeforeInit.toString()}`
    );
    // @ts-ignore
    const tokenInfoForCurve = await tokenCreatorProgram.account.tokenInfo.fetch(
      tokenInfoPda // Используем глобальную tokenInfoPda
    );
    const totalSupplyForCurve = BigInt(
      tokenInfoForCurve.totalSupply.toString()
    );
    const expectedBalance = (totalSupplyForCurve * BigInt(40)) / BigInt(100);

    console.log(
      `Expected balance for bonding curve (40%): ${expectedBalance.toString()}`
    );
    assert.equal(
      actualBalanceBeforeInit,
      expectedBalance,
      "Balance mismatch BEFORE calling initializeCurve"
    );

    try {
      const txSignature = await bondingCurveProgram.methods
        .initializeCurve()
        .accounts({
          bondingCurve: bondingCurvePda,
          mint: userTokenMint,
          nDollarMint: nDollarMint,
          bondingCurveTokenAccount: bondingCurveTokenAccount,
          authority: wallet.publicKey,
          rentPayer: wallet.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .rpc({ commitment: "confirmed" });

      await provider.connection.confirmTransaction(txSignature, "confirmed");

      // Проверяем, что ATA для nDollarTreasury действительно создан
      try {
        const ataInfo = await getAccount(provider.connection, nDollarTreasury);
        console.log(
          "nDollarTreasury ATA создан, owner:",
          ataInfo.owner.toBase58()
        );
      } catch (e) {
        console.error(
          "nDollarTreasury ATA не найден после initializeCurve!",
          e.message
        );
      }

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
      // @ts-ignore
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

  it("7b. Creator buys tokens по спец. цене (creator_buy_tokens)", async () => {
    await sleep(1000);
    console.log("\n7b: Creator buys tokens по спец. цене...");

    assert(bondingCurvePda, "bondingCurvePda not set");
    assert(userTokenMint, "userTokenMint not set");

    // --- Проверка и подготовка баланса для creator_buy_tokens ---
    const CREATOR_BUY_AMOUNT = new BN(10_000_000).mul(
      new BN(10 ** tokenDecimals)
    ); // 10M токенов
    const CREATOR_BUY_PRICE = 50_000; // 0.00005 * 1_000_000_000 = 50_000 лампортов за токен
    const CREATOR_BUY_TOTAL_NDOLLAR = new BN(500_000_000_000); // 500 N-Dollar (9 децималов)
    console.log("CREATOR_BUY_AMOUNT:", CREATOR_BUY_AMOUNT.toString());
    console.log("CREATOR_BUY_PRICE:", CREATOR_BUY_PRICE);
    console.log(
      "CREATOR_BUY_TOTAL_NDOLLAR:",
      CREATOR_BUY_TOTAL_NDOLLAR.toString()
    );
    let userNDollarBalance = await getTokenBalance(
      provider,
      userNDollarAccount
    );
    console.log(
      "User N-Dollar balance ДО creator_buy_tokens:",
      userNDollarBalance.toString()
    );
    if (
      BigInt(userNDollarBalance.toString()) <
      BigInt(CREATOR_BUY_TOTAL_NDOLLAR.toString())
    ) {
      const diff =
        BigInt(CREATOR_BUY_TOTAL_NDOLLAR.toString()) -
        BigInt(userNDollarBalance.toString());
      const solToSwap = new BN(Number(diff) * 2); // swap с запасом
      console.log(
        "Недостаточно N-Dollar для creator buy. Делаем swap SOL -> N-Dollar..."
      );
      await liquidityPoolProgram.methods
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
      userNDollarBalance = await getTokenBalance(provider, userNDollarAccount);
      console.log(
        "User N-Dollar balance после swap:",
        userNDollarBalance.toString()
      );
    }
    // assert.equal(
    //   userNDollarBalance.toString(),
    //   CREATOR_BUY_TOTAL_NDOLLAR.toString(),
    //   "На счету пользователя должно быть ровно 500_000_000_000 лампортов N-Dollar для creator buy"
    // );

    // Найти escrow PDA и ATA для escrow
    const [creatorEscrowPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("creator_lock"),
        userTokenMint.toBuffer(),
        wallet.publicKey.toBuffer(),
      ],
      bondingCurveProgram.programId
    );
    const creatorEscrowTokenAccount = getAssociatedTokenAddressSync(
      userTokenMint,
      creatorEscrowPda,
      true
    );

    // Создать ATA для escrow, если не существует
    let needCreateEscrowATA = false;
    try {
      await getAccount(provider.connection, creatorEscrowTokenAccount);
    } catch {
      needCreateEscrowATA = true;
    }
    if (needCreateEscrowATA) {
      const createATAIx = createAssociatedTokenAccountInstruction(
        wallet.publicKey, // payer
        creatorEscrowTokenAccount,
        creatorEscrowPda, // owner
        userTokenMint
      );
      const tx = new anchor.web3.Transaction().add(createATAIx);
      const sig = await provider.sendAndConfirm(tx);
      console.log("Created creatorEscrowTokenAccount ATA:", sig);
    }

    // Вызов creator_buy_tokens
    // Получаем свежий блокхэш для диагностики
    const latestBlockhash = await provider.connection.getLatestBlockhash(
      "confirmed"
    );
    console.log("Latest blockhash перед creator_buy_tokens:", latestBlockhash);
    console.log("Перед вызовом creator_buy_tokens");

    // Логируем децималы токенов (безопасно)
    function getDecimalsFromParsedAccountInfo(info: any): number | undefined {
      if (
        info &&
        info.value &&
        info.value.data &&
        typeof info.value.data === "object" &&
        "parsed" in info.value.data
      ) {
        return info.value.data.parsed?.info?.decimals;
      }
      return undefined;
    }
    const userTokenMintInfo = await provider.connection.getParsedAccountInfo(
      userTokenMint
    );
    const nDollarMintInfo = await provider.connection.getParsedAccountInfo(
      nDollarMint
    );
    const userTokenDecimals =
      getDecimalsFromParsedAccountInfo(userTokenMintInfo);
    const nDollarDecimals = getDecimalsFromParsedAccountInfo(nDollarMintInfo);
    console.log("userTokenMint decimals:", userTokenDecimals);
    console.log("nDollarMint decimals:", nDollarDecimals);

    try {
      const txSignature = await bondingCurveProgram.methods
        .creatorBuyTokens()
        .accounts({
          bondingCurve: bondingCurvePda,
          mint: userTokenMint,
          bondingCurveTokenAccount: bondingCurveTokenAccount,
          nDollarTreasury: nDollarTreasury,
          creator: wallet.publicKey,
          creatorNDollarAccount: userNDollarAccount,
          creatorEscrow: creatorEscrowPda,
          creatorEscrowTokenAccount: creatorEscrowTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc({ commitment: "confirmed" });
      console.log("creator_buy_tokens tx:", txSignature);
      await provider.connection.confirmTransaction(txSignature, "confirmed");
      // Логируем баланс после покупки по спец. цене
      const userNDollarBalanceAfter = await getTokenBalance(
        provider,
        userNDollarAccount
      );
      console.log(
        "User N-Dollar balance ПОСЛЕ creator_buy_tokens:",
        userNDollarBalanceAfter.toString()
      );
    } catch (error) {
      console.error("Ошибка при вызове creator_buy_tokens:", error);
      const latestBlockhashAfter = await provider.connection.getLatestBlockhash(
        "confirmed"
      );
      console.log("Latest blockhash после ошибки:", latestBlockhashAfter);
      throw error;
    }

    // Проверить статус
    // @ts-ignore
    const curveAccount = await bondingCurveProgram.account.bondingCurve.fetch(
      bondingCurvePda
    );
    console.log(
      "creator_buy_status после creator_buy_tokens:",
      curveAccount.creatorBuyStatus
    );
    assert.equal(
      curveAccount.creatorBuyStatus,
      1,
      "creator_buy_status должен быть Claimed (1)"
    );

    // Проверить escrow токены
    const escrowBalance = await getTokenBalance(
      provider,
      creatorEscrowTokenAccount
    );
    console.log("Escrow token balance:", escrowBalance.toString());
    assert(escrowBalance > 0, "Escrow должен содержать купленные токены");

    // Проверить, что списались N-Dollar
    const userNDollarBalanceAfter = await getTokenBalance(
      provider,
      userNDollarAccount
    );
    console.log(
      "User N-Dollar balance after creator buy:",
      userNDollarBalanceAfter.toString()
    );
    assert(
      userNDollarBalanceAfter < userNDollarBalance,
      "N-Dollar должен быть списан"
    );
  });

  it("7c. Unlocks creator tokens after lock period", async () => {
    await sleep(11000); // Ждём 11 секунд (lock период 10 сек)
    console.log("\n7c: Unlocking creator tokens...");

    // --- Получаем PDA и ATA для escrow ---
    const [creatorEscrowPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("creator_lock"),
        userTokenMint.toBuffer(),
        wallet.publicKey.toBuffer(),
      ],
      bondingCurveProgram.programId
    );
    const creatorEscrowTokenAccount = getAssociatedTokenAddressSync(
      userTokenMint,
      creatorEscrowPda,
      true
    );

    // --- Проверяем балансы до ---
    const escrowBalanceBefore = await getTokenBalance(
      provider,
      creatorEscrowTokenAccount
    );
    const userBalanceBefore = await getTokenBalance(provider, userTokenAccount);
    console.log("Escrow до:", escrowBalanceBefore.toString());
    console.log("User до:", userBalanceBefore.toString());

    // --- Вызываем unlock_creator_tokens ---
    const txSignature = await bondingCurveProgram.methods
      .unlockCreatorTokens()
      .accounts({
        bondingCurve: bondingCurvePda,
        mint: userTokenMint,
        recipient: wallet.publicKey,
        creatorEscrow: creatorEscrowPda,
        creatorEscrowTokenAccount: creatorEscrowTokenAccount,
        recipientTokenAccount: userTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc({ commitment: "confirmed" });
    console.log("Unlock creator tokens tx:", txSignature);
    await provider.connection.confirmTransaction(txSignature, "confirmed");

    // --- Проверяем балансы после ---
    const escrowBalanceAfter = await getTokenBalance(
      provider,
      creatorEscrowTokenAccount
    );
    const userBalanceAfter = await getTokenBalance(provider, userTokenAccount);
    console.log("Escrow после:", escrowBalanceAfter.toString());
    console.log("User после:", userBalanceAfter.toString());

    // --- Проверки ---
    assert.equal(
      escrowBalanceAfter,
      BigInt(0),
      "Escrow должен быть пуст после unlock"
    );
    assert(
      userBalanceAfter > userBalanceBefore,
      "User должен получить токены после unlock"
    );
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
    // @ts-ignore
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
        .buy(amountToBuy, amountToBuy) // maxTotalCost = amountToBuy для простоты
        .accounts({
          bondingCurve: bondingCurvePda,
          mint: userTokenMint,
          nDollarMint: nDollarMint,
          bondingCurveTokenAccount: bondingCurveTokenAccount,
          nDollarTreasury: nDollarTreasury,
          userTokenAccount: userTokenAccount,
          userNDollarAccount: userNDollarAccount,
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
    // @ts-ignore
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
    const minTotalReturn = new BN(0);
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
        .sell(amountToSell, minTotalReturn)
        .accounts({
          bondingCurve: bondingCurvePda,
          mint: userTokenMint,
          nDollarMint: nDollarMint,
          bondingCurveTokenAccount: bondingCurveTokenAccount,
          nDollarTreasury: nDollarTreasury,
          userTokenAccount: userTokenAccount,
          userNDollarAccount: userNDollarAccount,
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
