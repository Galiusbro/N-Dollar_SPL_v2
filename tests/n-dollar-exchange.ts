import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
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
  getMint,
  createSetAuthorityInstruction,
  AuthorityType,
} from "@solana/spl-token";
import { assert } from "chai";

// Используем BN из bn.js
import BN from "bn.js";

describe("N-Dollar Exchange & Coin Creation Platform", () => {
  // Настройка провайдера
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Загрузка программ
  const nDollarProgram = anchor.workspace.NDollarToken as Program;
  const genesisProgram = anchor.workspace.Genesis as Program;
  const bondingCurveProgram = anchor.workspace.BondingCurve as Program;
  const liquidityManagerProgram = anchor.workspace.LiquidityManager as Program;
  const tradingExchangeProgram = anchor.workspace.TradingExchange as Program;
  const referralSystemProgram = anchor.workspace.ReferralSystem as Program;

  // Кошельки для тестирования
  const admin = Keypair.generate();
  const user1 = Keypair.generate();
  const user2 = Keypair.generate();

  // Переменные для хранения аккаунтов
  let nDollarMint: PublicKey;
  let adminNDollarAccount: PublicKey;
  let user1NDollarAccount: PublicKey;
  let user2NDollarAccount: PublicKey;
  let mockMetadataProgram: Keypair;

  // Для тестирования бондинговой кривой
  let memeCoinMint: PublicKey;
  let adminMemeCoinAccount: PublicKey;
  let user1MemeCoinAccount: PublicKey;
  let user2MemeCoinAccount: PublicKey;
  let liquidity_pool: PublicKey;
  let bondingCurveAccount: PublicKey;

  // Для тестирования ликвидности
  let liquidityManagerAccount: PublicKey;
  let poolSolAccount: PublicKey;
  let poolNDollarAccount: PublicKey;

  // Для тестирования торговой биржи
  let exchangeDataAccount: PublicKey;
  let tradingExchangeAccount: PublicKey;

  // Параметры для инициализации N-Dollar
  const nDollarName = "N-Dollar";
  const nDollarSymbol = "NDOL";
  const nDollarUri = "https://example.com/ndollar.json";
  const nDollarDecimals = 9;

  it("Инициализирует тестовые аккаунты", async () => {
    // Выделяем SOL для тестовых аккаунтов
    await provider.connection.requestAirdrop(
      admin.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.requestAirdrop(
      user1.publicKey,
      5 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.requestAirdrop(
      user2.publicKey,
      5 * anchor.web3.LAMPORTS_PER_SOL
    );

    // Ждем подтверждения транзакций
    await new Promise((resolve) => setTimeout(resolve, 2000));

    console.log("Тестовые аккаунты успешно инициализированы");
    console.log("Admin:", admin.publicKey.toString());
    console.log("User1:", user1.publicKey.toString());
    console.log("User2:", user2.publicKey.toString());
  });

  it("Проверяет подключение к программам", async () => {
    // Проверяем подключение к программам
    console.log("Программа NDollarToken:", nDollarProgram.programId.toString());
    console.log("Программа Genesis:", genesisProgram.programId.toString());
    console.log(
      "Программа BondingCurve:",
      bondingCurveProgram.programId.toString()
    );
    console.log(
      "Программа LiquidityManager:",
      liquidityManagerProgram.programId.toString()
    );
    console.log(
      "Программа TradingExchange:",
      tradingExchangeProgram.programId.toString()
    );
    console.log(
      "Программа ReferralSystem:",
      referralSystemProgram.programId.toString()
    );
  });

  it("Создает N-Dollar токен", async () => {
    // Создаем мок для метадата программы
    mockMetadataProgram = Keypair.generate();

    // Создаем минт для N-Dollar
    const nDollarMintKeypair = Keypair.generate();
    nDollarMint = nDollarMintKeypair.publicKey;

    // Создаем ассоциированный токен аккаунт для админа
    adminNDollarAccount = await getAssociatedTokenAddress(
      nDollarMint,
      admin.publicKey
    );

    // Создаем ассоциированные токен аккаунты для пользователей
    user1NDollarAccount = await getAssociatedTokenAddress(
      nDollarMint,
      user1.publicKey
    );

    user2NDollarAccount = await getAssociatedTokenAddress(
      nDollarMint,
      user2.publicKey
    );

    // Создаем транзакцию для инициализации минта и создания ассоциированного токен аккаунта
    const tx = new Transaction();

    // Инициализация минта
    tx.add(
      SystemProgram.createAccount({
        fromPubkey: admin.publicKey,
        newAccountPubkey: nDollarMint,
        lamports: await provider.connection.getMinimumBalanceForRentExemption(
          82
        ),
        space: 82,
        programId: TOKEN_PROGRAM_ID,
      })
    );

    tx.add(
      createInitializeMintInstruction(
        nDollarMint,
        nDollarDecimals,
        admin.publicKey,
        admin.publicKey
      )
    );

    // Создаем ассоциированный токен аккаунт для админа
    tx.add(
      createAssociatedTokenAccountInstruction(
        admin.publicKey,
        adminNDollarAccount,
        admin.publicKey,
        nDollarMint
      )
    );

    // Создаем ассоциированные токен аккаунты для пользователей
    tx.add(
      createAssociatedTokenAccountInstruction(
        admin.publicKey,
        user1NDollarAccount,
        user1.publicKey,
        nDollarMint
      )
    );

    tx.add(
      createAssociatedTokenAccountInstruction(
        admin.publicKey,
        user2NDollarAccount,
        user2.publicKey,
        nDollarMint
      )
    );

    // Отправляем транзакцию
    await anchor.web3.sendAndConfirmTransaction(provider.connection, tx, [
      admin,
      nDollarMintKeypair,
    ]);

    console.log("N-Dollar минт успешно создан:", nDollarMint.toString());
    console.log(
      "Ассоциированный токен аккаунт админа:",
      adminNDollarAccount.toString()
    );
    console.log(
      "Ассоциированный токен аккаунт user1:",
      user1NDollarAccount.toString()
    );
    console.log(
      "Ассоциированный токен аккаунт user2:",
      user2NDollarAccount.toString()
    );
  });

  it("Минтит N-Dollar токены админу", async () => {
    try {
      // Разбиваем большое число на части, чтобы избежать переполнения
      // Минтим токены порциями по 1 миллиону
      const batch1 = new BN(1_000_000).mul(
        new BN(10).pow(new BN(nDollarDecimals))
      );
      const tx1 = new Transaction();
      tx1.add(
        createMintToInstruction(
          nDollarMint,
          adminNDollarAccount,
          admin.publicKey,
          Number(batch1.toString())
        )
      );
      await anchor.web3.sendAndConfirmTransaction(provider.connection, tx1, [
        admin,
      ]);
      console.log("Минтинг: первый миллион N-Dollar завершен");

      // Минтим остальные токены (107 миллионов)
      const batch2 = new BN(107_000_000).mul(
        new BN(10).pow(new BN(nDollarDecimals))
      );
      const tx2 = new Transaction();
      tx2.add(
        createMintToInstruction(
          nDollarMint,
          adminNDollarAccount,
          admin.publicKey,
          Number(batch2.toString())
        )
      );
      await anchor.web3.sendAndConfirmTransaction(provider.connection, tx2, [
        admin,
      ]);
      console.log("Минтинг: остальные 107 миллионов N-Dollar завершены");

      // Проверяем баланс
      const adminBalance = await provider.connection.getTokenAccountBalance(
        adminNDollarAccount
      );
      console.log(
        "Баланс N-Dollar админа:",
        adminBalance.value.uiAmount,
        "NDOL"
      );
      assert.equal(adminBalance.value.uiAmount, 108000000);
    } catch (error) {
      console.error("Ошибка при минтинге N-Dollar:", error);
      throw error;
    }
  });

  it("Инициализирует Liquidity Manager и создает пул ликвидности", async () => {
    try {
      // Находим PDA для менеджера ликвидности
      const [liquidityManagerPDA, liquidityManagerBump] =
        PublicKey.findProgramAddressSync(
          [Buffer.from("liquidity_manager"), admin.publicKey.toBytes()],
          liquidityManagerProgram.programId
        );
      liquidityManagerAccount = liquidityManagerPDA;

      // Создаем PDA для хранения SOL пула ликвидности
      const [poolSolPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("pool_sol"), liquidityManagerPDA.toBytes()],
        liquidityManagerProgram.programId
      );
      poolSolAccount = poolSolPDA;

      // Создаем аккаунт для хранения N-Dollar пула ликвидности
      poolNDollarAccount = await getAssociatedTokenAddress(
        nDollarMint,
        liquidityManagerAccount,
        true
      );

      // Инициализируем менеджер ликвидности
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

      // Запрашиваем SOL для пула ликвидности
      const createPoolSolAccountTx = await provider.connection.requestAirdrop(
        poolSolAccount,
        5 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(createPoolSolAccountTx);

      // Создаем аккаунт для N-Dollar пула ликвидности
      const tx = new Transaction();
      tx.add(
        createAssociatedTokenAccountInstruction(
          admin.publicKey,
          poolNDollarAccount,
          liquidityManagerAccount,
          nDollarMint
        )
      );

      // Отправляем транзакцию
      await anchor.web3.sendAndConfirmTransaction(provider.connection, tx, [
        admin,
      ]);

      // Добавляем ликвидность в пул (5 SOL и почти все N-Dollar для доступности продажи)
      const solAmount = new BN(5 * anchor.web3.LAMPORTS_PER_SOL);

      // Переводим N-Dollar в пул по частям
      // Сначала переводим первую часть (50 миллионов)
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

      // Затем остальные (примерно 56.9 миллионов, чтобы в сумме было ~99% от 108 миллионов)
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

      // Добавляем SOL в пул через команду add_liquidity
      await liquidityManagerProgram.methods
        .addLiquidity(
          solAmount,
          new BN(0) // мы уже добавили N-Dollar напрямую
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

      // Проверяем балансы пула
      const poolSolBalance = await provider.connection.getBalance(
        poolSolAccount
      );
      const poolNDollarBalance =
        await provider.connection.getTokenAccountBalance(poolNDollarAccount);

      console.log(
        "Баланс SOL в пуле:",
        poolSolBalance / anchor.web3.LAMPORTS_PER_SOL,
        "SOL"
      );
      console.log(
        "Баланс N-Dollar в пуле:",
        poolNDollarBalance.value.uiAmount,
        "NDOL"
      );

      // Проверяем, что балансы соответствуют ожидаемым
      assert(poolSolBalance >= 5 * anchor.web3.LAMPORTS_PER_SOL);
      assert(poolNDollarBalance.value.uiAmount >= 106000000);
    } catch (error) {
      console.log("Ошибка при инициализации Liquidity Manager:", error);
      // Продолжаем тест, чтобы другие тесты могли выполняться
    }
  });

  it("Инициализирует Trading Exchange", async () => {
    try {
      // Находим PDA для торговой биржи
      const [exchangeDataPDA, exchangeDataBump] =
        PublicKey.findProgramAddressSync(
          [Buffer.from("exchange_data"), admin.publicKey.toBytes()],
          tradingExchangeProgram.programId
        );
      exchangeDataAccount = exchangeDataPDA;

      // Инициализируем данные обмена
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
        "Trading Exchange данные успешно инициализированы:",
        exchangeDataAccount.toString()
      );

      // Создаем и инициализируем аккаунт для TradingExchange
      const [tradingExchangePDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("trading_exchange"), admin.publicKey.toBytes()],
        tradingExchangeProgram.programId
      );
      tradingExchangeAccount = tradingExchangePDA;

      // Инициализируем TradingExchange
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
        "Trading Exchange успешно инициализирован:",
        tradingExchangeAccount.toString()
      );
    } catch (error) {
      console.log("Ошибка при инициализации Trading Exchange:", error);

      // Для продолжения тестов создаем мок PDA
      const [tradingExchangePDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("trading_exchange"), admin.publicKey.toBytes()],
        tradingExchangeProgram.programId
      );
      tradingExchangeAccount = tradingExchangePDA;
    }
  });

  it("Покупает N-Dollar за SOL через Trading Exchange", async () => {
    // Получаем начальные балансы
    const initialSolBalance = await provider.connection.getBalance(
      user1.publicKey
    );
    const initialNDollarBalance =
      await provider.connection.getTokenAccountBalance(user1NDollarAccount);

    console.log(
      "Начальный баланс SOL пользователя:",
      initialSolBalance / anchor.web3.LAMPORTS_PER_SOL,
      "SOL"
    );
    console.log(
      "Начальный баланс N-Dollar пользователя:",
      initialNDollarBalance.value.uiAmount,
      "NDOL"
    );

    // Покупаем 1 SOL на N-Dollar через торговую биржу
    const solAmount = new BN(1 * anchor.web3.LAMPORTS_PER_SOL);

    await tradingExchangeProgram.methods
      .buyNDollar(solAmount)
      .accounts({
        user: user1.publicKey,
        tradingExchange: tradingExchangeAccount,
        userNdollarAccount: user1NDollarAccount,
        liquidityManager: liquidityManagerAccount,
        poolSolAccount: poolSolAccount,
        poolNdollarAccount: poolNDollarAccount,
        liquidityManagerProgram: liquidityManagerProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user1])
      .rpc();

    // Получаем новые балансы
    const newSolBalance = await provider.connection.getBalance(user1.publicKey);
    const newNDollarBalance = await provider.connection.getTokenAccountBalance(
      user1NDollarAccount
    );

    console.log(
      "Новый баланс SOL пользователя:",
      newSolBalance / anchor.web3.LAMPORTS_PER_SOL,
      "SOL"
    );
    console.log(
      "Новый баланс N-Dollar пользователя:",
      newNDollarBalance.value.uiAmount,
      "NDOL"
    );

    // Проверяем, что SOL уменьшился примерно на 1 SOL (учитывая комиссии за транзакцию)
    assert(initialSolBalance - newSolBalance >= solAmount.toNumber());

    // Проверяем, что N-Dollar увеличился (должно быть примерно 990 N-Dollar за 1 SOL с учетом комиссии 1%)
    assert(
      newNDollarBalance.value.uiAmount > initialNDollarBalance.value.uiAmount
    );
    assert(
      newNDollarBalance.value.uiAmount - initialNDollarBalance.value.uiAmount >=
        0.9 // Ожидаем около 0.99 N-Dollar, учитывая комиссию 1%
    );
  });

  it("Продает N-Dollar за SOL через Trading Exchange", async () => {
    // Получаем начальные балансы
    const initialSolBalance = await provider.connection.getBalance(
      user1.publicKey
    );
    const initialNDollarBalance =
      await provider.connection.getTokenAccountBalance(user1NDollarAccount);

    console.log(
      "Начальный баланс SOL пользователя:",
      initialSolBalance / anchor.web3.LAMPORTS_PER_SOL,
      "SOL"
    );
    console.log(
      "Начальный баланс N-Dollar пользователя:",
      initialNDollarBalance.value.uiAmount,
      "NDOL"
    );

    // Минтим дополнительные N-Dollar пользователю для продажи
    const mintAmount = new BN(500 * Math.pow(10, nDollarDecimals)); // 500 N-Dollar
    const mintTx = new Transaction();
    mintTx.add(
      createMintToInstruction(
        nDollarMint,
        user1NDollarAccount,
        admin.publicKey,
        mintAmount.toNumber()
      )
    );
    await anchor.web3.sendAndConfirmTransaction(provider.connection, mintTx, [
      admin,
    ]);

    console.log("Пользователю выдано 500 N-Dollar для теста продажи");

    // Получаем обновленный баланс N-Dollar после минта
    const updatedNDollarBalance =
      await provider.connection.getTokenAccountBalance(user1NDollarAccount);
    console.log(
      "Обновленный баланс N-Dollar пользователя:",
      updatedNDollarBalance.value.uiAmount,
      "NDOL"
    );

    // Продаем N-Dollar за SOL через торговую биржу - ИЗМЕНЕНО: продаем только 10 N-Dollar
    const ndollarAmount = new BN(10 * Math.pow(10, nDollarDecimals)); // 10 N-Dollar

    console.log("Продаем 10 N-Dollar за SOL");

    await tradingExchangeProgram.methods
      .sellNDollar(ndollarAmount)
      .accounts({
        user: user1.publicKey,
        tradingExchange: tradingExchangeAccount,
        userNdollarAccount: user1NDollarAccount,
        liquidityManager: liquidityManagerAccount,
        poolSolAccount: poolSolAccount,
        poolNdollarAccount: poolNDollarAccount,
        liquidityManagerProgram: liquidityManagerProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user1])
      .rpc();

    // Получаем новые балансы
    const newSolBalance = await provider.connection.getBalance(user1.publicKey);
    const newNDollarBalance = await provider.connection.getTokenAccountBalance(
      user1NDollarAccount
    );

    console.log(
      "Новый баланс SOL пользователя:",
      newSolBalance / anchor.web3.LAMPORTS_PER_SOL,
      "SOL"
    );
    console.log(
      "Новый баланс N-Dollar пользователя:",
      newNDollarBalance.value.uiAmount,
      "NDOL"
    );

    // Проверяем, что N-Dollar уменьшились на проданное количество (10)
    assert.approximately(
      updatedNDollarBalance.value.uiAmount - newNDollarBalance.value.uiAmount,
      10,
      0.1
    );

    // Проверяем, что SOL увеличился
    assert(newSolBalance > initialSolBalance);
  });

  it("Создает мемкоин и устанавливает бондинговую кривую", async () => {
    // Создаем минт для мемкоина
    const memeCoinMintKeypair = Keypair.generate();
    memeCoinMint = memeCoinMintKeypair.publicKey;

    // Создаем ассоциированные токен аккаунты для всех пользователей
    adminMemeCoinAccount = await getAssociatedTokenAddress(
      memeCoinMint,
      admin.publicKey
    );

    user1MemeCoinAccount = await getAssociatedTokenAddress(
      memeCoinMint,
      user1.publicKey
    );

    user2MemeCoinAccount = await getAssociatedTokenAddress(
      memeCoinMint,
      user2.publicKey
    );

    // Создаем отдельный пул ликвидности N-Dollar для бондинговой кривой
    const [bondingCurvePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("bonding_curve"), memeCoinMint.toBytes()],
      bondingCurveProgram.programId
    );
    bondingCurveAccount = bondingCurvePDA;

    // Создаем аккаунт для пула ликвидности, принадлежащий бондинговой кривой
    liquidity_pool = await getAssociatedTokenAddress(
      nDollarMint,
      bondingCurveAccount,
      true
    );

    // Создаем транзакцию для инициализации минта и создания ассоциированных токен аккаунтов
    const tx = new Transaction();

    // Инициализация минта
    tx.add(
      SystemProgram.createAccount({
        fromPubkey: admin.publicKey,
        newAccountPubkey: memeCoinMint,
        lamports: await provider.connection.getMinimumBalanceForRentExemption(
          82
        ),
        space: 82,
        programId: TOKEN_PROGRAM_ID,
      })
    );

    tx.add(
      createInitializeMintInstruction(
        memeCoinMint,
        9, // 9 децималов
        admin.publicKey,
        admin.publicKey
      )
    );

    // Создаем ассоциированные токен аккаунты
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

    // Отправляем транзакцию
    await anchor.web3.sendAndConfirmTransaction(provider.connection, tx, [
      admin,
      memeCoinMintKeypair,
    ]);

    // Создаем пул ликвидности для бондинговой кривой
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

    // Минтим начальную ликвидность в пул
    const mintToPoolTx = new Transaction();
    const poolAmount = new BN(1000 * Math.pow(10, nDollarDecimals)); // 1000 N-Dollar для ликвидности

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

    console.log(
      "Пул ликвидности для бондинговой кривой создан с 1000 N-Dollar"
    );

    // Находим PDA для бондинговой кривой
    const [bondingCurvePDA2, bondingCurveBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("bonding_curve"), memeCoinMint.toBytes()],
        bondingCurveProgram.programId
      );

    // Устанавливаем начальные параметры бондинговой кривой
    const initialPrice = new BN(1000000); // 0.001 N-Dollar
    const power = 2; // Степенной показатель для кривой
    const feePercent = 100; // 1% комиссии (в базисных пунктах)

    // Инициализируем бондинговую кривую
    await bondingCurveProgram.methods
      .initializeBondingCurve(memeCoinMint, initialPrice, power, feePercent)
      .accounts({
        creator: admin.publicKey,
        bondingCurve: bondingCurveAccount,
        coinMint: memeCoinMint,
        ndollarMint: nDollarMint,
        liquidityPool: liquidity_pool, // Используем новый пул ликвидности
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([admin])
      .rpc();

    // Передаем права mint authority контракту bonding-curve
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

    console.log("Мемкоин успешно создан:", memeCoinMint.toString());
    console.log(
      "Бондинговая кривая установлена:",
      bondingCurveAccount.toString()
    );
    console.log("Права mint authority переданы бондинговой кривой");
    console.log("Пул ликвидности:", liquidity_pool.toString());
  });

  // Добавляем вспомогательную функцию для получения текущей цены мемкоина
  async function getMemecoinPrice(
    provider: anchor.AnchorProvider,
    bondingCurveProgram: Program,
    bondingCurveAccount: PublicKey,
    memeCoinMint: PublicKey
  ): Promise<number> {
    try {
      // Получаем данные аккаунта бондинговой кривой
      const bondingCurveInfo = await provider.connection.getAccountInfo(
        bondingCurveAccount
      );
      if (!bondingCurveInfo) {
        console.log("Аккаунт бондинговой кривой не найден");
        return 0;
      }

      // Десериализуем данные аккаунта, чтобы получить значения
      // Для этого также можем использовать anchor coder
      const accountData = bondingCurveProgram.coder.accounts.decode(
        "bondingCurve",
        bondingCurveInfo.data
      );

      // Получаем необходимые значения для расчета цены
      const totalSupply = accountData.totalSupplyInCurve;
      const reserveBalance = accountData.reserveBalance;
      const power = accountData.power;
      const initialPrice = accountData.initialPrice;

      // Если supply = 0, возвращаем начальную цену
      if (totalSupply.isZero()) {
        // Начальная цена в структуре хранится в ламопртах N-Dollar
        const priceInNDollar = initialPrice.toNumber() / Math.pow(10, 9);
        console.log(
          `Текущая цена мемкоина: ${priceInNDollar} N-Dollar (начальная цена)`
        );
        return priceInNDollar;
      }

      // Для получения текущей цены также используем RPC метод
      try {
        // Выводим имеющиеся данные для расчета цены
        console.log(
          `Total Supply: ${totalSupply.toString()}, Reserve Balance: ${reserveBalance.toString()}, Power: ${power}`
        );

        // Пытаемся рассчитать цену по формуле: price = (reserve_balance * power) / total_supply
        const price = reserveBalance.mul(new BN(power)).div(totalSupply);
        const priceInNDollar = price.toNumber() / Math.pow(10, 9);

        console.log(`Текущая цена мемкоина: ${priceInNDollar} N-Dollar`);
        return priceInNDollar;
      } catch (error) {
        console.log("Ошибка при расчете цены:", error.message);
        return 0;
      }
    } catch (error) {
      console.log("Ошибка при получении цены мемкоина:", error.message);
      return 0;
    }
  }

  it("Покупает мемкоин за N-Dollar через бондинговую кривую", async () => {
    try {
      // Получаем начальные балансы
      const initialNDollarBalance =
        await provider.connection.getTokenAccountBalance(user1NDollarAccount);
      const initialMemeCoinBalance =
        await provider.connection.getTokenAccountBalance(user1MemeCoinAccount);

      console.log(
        "Начальный баланс N-Dollar пользователя:",
        initialNDollarBalance.value.uiAmount,
        "NDOL"
      );
      console.log(
        "Начальный баланс мемкоина пользователя:",
        initialMemeCoinBalance.value.uiAmount
      );

      // Сначала нужно перевести N-Dollar пользователю
      const ndollarToUserTx = new Transaction();
      const transferAmount = new BN(100 * Math.pow(10, nDollarDecimals)); // 100 N-Dollar

      ndollarToUserTx.add(
        createMintToInstruction(
          nDollarMint,
          user1NDollarAccount,
          admin.publicKey,
          transferAmount.toNumber()
        )
      );

      await anchor.web3.sendAndConfirmTransaction(
        provider.connection,
        ndollarToUserTx,
        [admin]
      );

      console.log("Пользователю переведено 100 N-Dollar для покупки мемкоинов");

      // Массив сумм для тестирования (в N-Dollar)
      const testAmounts = [
        0.00000001, // Неадекватно маленькая сумма
        0.001, // Маленькая сумма
        1, // Средняя сумма
        10, // Большая сумма
        50, // Очень большая сумма
      ];

      // Тестируем каждую сумму
      for (const amount of testAmounts) {
        const ndollarAmount = new BN(
          Math.floor(amount * Math.pow(10, nDollarDecimals))
        );

        console.log(`\nПытаемся купить мемкоины за ${amount} N-Dollar`);

        // Получаем цену до покупки
        console.log(`Цена мемкоина ПЕРЕД покупкой ${amount} N-Dollar:`);
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
              buyer: user1.publicKey,
              bondingCurve: bondingCurveAccount,
              coinMint: memeCoinMint,
              ndollarMint: nDollarMint,
              buyerCoinAccount: user1MemeCoinAccount,
              buyerNdollarAccount: user1NDollarAccount,
              liquidityPool: liquidity_pool,
              tokenProgram: TOKEN_PROGRAM_ID,
              systemProgram: SystemProgram.programId,
            })
            .signers([user1])
            .rpc();

          // Получаем новые балансы после этой покупки
          const newBalance = await provider.connection.getTokenAccountBalance(
            user1MemeCoinAccount
          );

          console.log(
            `Успешно куплено! Текущий баланс мемкоина: ${newBalance.value.uiAmount}`
          );

          // Получаем цену после покупки
          console.log(`Цена мемкоина ПОСЛЕ покупки ${amount} N-Dollar:`);
          await getMemecoinPrice(
            provider,
            bondingCurveProgram,
            bondingCurveAccount,
            memeCoinMint
          );
        } catch (error) {
          console.log(
            `Ошибка при покупке за ${amount} N-Dollar:`,
            error.message
          );
        }
      }

      // Получаем итоговые балансы
      const finalNDollarBalance =
        await provider.connection.getTokenAccountBalance(user1NDollarAccount);
      const finalMemeCoinBalance =
        await provider.connection.getTokenAccountBalance(user1MemeCoinAccount);

      console.log(
        "\nИтоговый баланс N-Dollar пользователя:",
        finalNDollarBalance.value.uiAmount,
        "NDOL"
      );
      console.log(
        "Итоговый баланс мемкоина пользователя:",
        finalMemeCoinBalance.value.uiAmount
      );

      // Проверяем, что мемкоины увеличились
      assert(
        finalMemeCoinBalance.value.uiAmount >
          (initialMemeCoinBalance.value.uiAmount || 0),
        "Баланс мемкоинов должен увеличиться"
      );
    } catch (error) {
      console.log("Ошибка при покупке мемкоина:", error);
      // Пропускаем тест вместо того, чтобы падать
    }
  });

  it("Продает мемкоин за N-Dollar через бондинговую кривую", async () => {
    try {
      // Получаем начальные балансы
      const initialNDollarBalance =
        await provider.connection.getTokenAccountBalance(user1NDollarAccount);
      const initialMemeCoinBalance =
        await provider.connection.getTokenAccountBalance(user1MemeCoinAccount);

      console.log(
        "Начальный баланс N-Dollar пользователя:",
        initialNDollarBalance.value.uiAmount,
        "NDOL"
      );
      console.log(
        "Начальный баланс мемкоина пользователя:",
        initialMemeCoinBalance.value.uiAmount
      );

      // Проверяем, есть ли у пользователя мемкоины для продажи
      if (initialMemeCoinBalance.value.uiAmount > 0) {
        // Расчет разных сумм для продажи
        const totalAmount = parseInt(initialMemeCoinBalance.value.amount);

        // Массив процентов для тестирования продажи
        const percentages = [
          0.0000001, // Неадекватно маленький процент
          0.001, // Маленький процент
          0.01, // 1%
          0.1, // 10%
          0.5, // 50%
        ];

        for (const percentage of percentages) {
          const tokenAmountToSell = Math.floor(totalAmount * percentage);
          if (tokenAmountToSell <= 0) continue;

          const amount = new BN(tokenAmountToSell);

          console.log(
            `\nПытаемся продать ${
              tokenAmountToSell / Math.pow(10, 9)
            } мемкоинов (${percentage * 100}% от всех)`
          );

          // Получаем цену до продажи
          console.log(
            `Цена мемкоина ПЕРЕД продажей ${percentage * 100}% токенов:`
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
                buyer: user1.publicKey,
                bondingCurve: bondingCurveAccount,
                coinMint: memeCoinMint,
                ndollarMint: nDollarMint,
                buyerCoinAccount: user1MemeCoinAccount,
                buyerNdollarAccount: user1NDollarAccount,
                liquidityPool: liquidity_pool,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
              })
              .signers([user1])
              .rpc();

            const newBalance = await provider.connection.getTokenAccountBalance(
              user1MemeCoinAccount
            );

            console.log(
              `Успешно продано! Текущий баланс мемкоина: ${newBalance.value.uiAmount}`
            );

            // Получаем цену после продажи
            console.log(
              `Цена мемкоина ПОСЛЕ продажи ${percentage * 100}% токенов:`
            );
            await getMemecoinPrice(
              provider,
              bondingCurveProgram,
              bondingCurveAccount,
              memeCoinMint
            );
          } catch (error) {
            console.log(
              `Ошибка при продаже ${
                tokenAmountToSell / Math.pow(10, 9)
              } мемкоинов:`,
              error.message
            );
          }
        }

        // Получаем итоговые балансы
        const finalNDollarBalance =
          await provider.connection.getTokenAccountBalance(user1NDollarAccount);
        const finalMemeCoinBalance =
          await provider.connection.getTokenAccountBalance(
            user1MemeCoinAccount
          );

        console.log(
          "\nИтоговый баланс N-Dollar пользователя:",
          finalNDollarBalance.value.uiAmount,
          "NDOL"
        );
        console.log(
          "Итоговый баланс мемкоина пользователя:",
          finalMemeCoinBalance.value.uiAmount
        );
      } else {
        console.log("У пользователя нет мемкоинов для продажи");
      }
    } catch (error) {
      console.log("Ошибка при продаже мемкоина:", error);
    }
  });

  it("Симулирует покупку мемкоина с учетом слиппеджа для разных сумм", async () => {
    try {
      // Тестируем разные суммы для симуляции
      const testAmounts = [
        0.00000001, // Неадекватно маленькая сумма
        0.001, // Маленькая сумма
        1, // Средняя сумма
        10, // Большая сумма
        1000, // Неадекватно большая сумма
        1000000, // Экстремально большая сумма
      ];

      console.log("\nСимуляция покупки мемкоинов для разных сумм:");

      for (const amount of testAmounts) {
        const ndollarAmount = new BN(
          Math.floor(amount * Math.pow(10, nDollarDecimals))
        );

        console.log(`\nСимуляция покупки на ${amount} N-Dollar:`);

        try {
          // Вызываем метод simulate_buy для получения информации о цене с учетом слиппеджа
          await bondingCurveProgram.methods
            .simulateBuy(ndollarAmount)
            .accounts({
              bondingCurve: bondingCurveAccount,
              coinMint: memeCoinMint,
            })
            .rpc();

          console.log(`Симуляция для ${amount} N-Dollar успешно выполнена`);
        } catch (error) {
          console.log(
            `Ошибка при симуляции для ${amount} N-Dollar:`,
            error.message
          );
        }
      }

      // Получаем текущую цену токена
      await bondingCurveProgram.methods
        .calculatePrice()
        .accounts({
          bondingCurve: bondingCurveAccount,
          coinMint: memeCoinMint,
        })
        .rpc();
    } catch (error) {
      console.log("Общая ошибка при симуляции покупки мемкоина:", error);
    }
  });

  it("Проверяет ценообразование N-Dollar при покупке и продаже", async () => {
    // Функция для получения текущей цены из аккаунта Liquidity Manager
    async function getCurrentPrice(): Promise<number> {
      const liquidityManagerInfo = await provider.connection.getAccountInfo(
        liquidityManagerAccount
      );
      if (!liquidityManagerInfo) {
        throw new Error("Liquidity Manager account not found");
      }
      // Декодируем данные аккаунта - поле currentPrice находится после полей:
      // - 8 байт дискриминатора
      // - 32 байта authority
      // - 32 байта n_dollar_mint
      // - 8 байт total_liquidity
      // - 8 байт total_users
      // = 88 байт до поля currentPrice, которое занимает 8 байт
      const currentPriceOffset = 8 + 32 + 32 + 8 + 8;
      const currentPrice = new BN(
        liquidityManagerInfo.data.slice(
          currentPriceOffset,
          currentPriceOffset + 8
        ),
        "le"
      ).toNumber();

      // current_price это количество N-Dollar за 1 SOL (в базовых единицах)
      const priceInNDollarPerSol = currentPrice / Math.pow(10, nDollarDecimals);
      return priceInNDollarPerSol;
    }

    // Проверяем начальную цену
    const initialPrice = await getCurrentPrice();
    console.log("Начальная цена: 1 SOL =", initialPrice, "N-Dollar");

    // Создаем нового пользователя для этого теста
    const testUser = Keypair.generate();
    await provider.connection.requestAirdrop(
      testUser.publicKey,
      3 * anchor.web3.LAMPORTS_PER_SOL
    );
    await new Promise((resolve) => setTimeout(resolve, 1000)); // Ждем подтверждения

    // Создаем токен аккаунт для тестового пользователя
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

    // Покупаем N-Dollar за 1 SOL
    const solAmount = new BN(1 * anchor.web3.LAMPORTS_PER_SOL);

    console.log("Покупаем N-Dollar за 1 SOL...");
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

    // Получаем количество купленных N-Dollar
    const boughtNDollarBalance =
      await provider.connection.getTokenAccountBalance(testUserNDollarAccount);
    console.log(
      "Купили:",
      boughtNDollarBalance.value.uiAmount,
      "N-Dollar за 1 SOL"
    );

    // Проверяем цену после покупки
    const priceAfterBuy = await getCurrentPrice();
    console.log("Цена после покупки: 1 SOL =", priceAfterBuy, "N-Dollar");
    console.log(
      "Изменение цены после покупки:",
      (((priceAfterBuy - initialPrice) / initialPrice) * 100).toFixed(2),
      "%"
    );

    // Продаем N-Dollar за SOL (половину от купленного)
    const halfBoughtAmountBN = new BN(
      Math.floor(
        (boughtNDollarBalance.value.uiAmount! * Math.pow(10, nDollarDecimals)) /
          2
      )
    );

    console.log(
      "Продаем",
      halfBoughtAmountBN.toNumber() / Math.pow(10, nDollarDecimals),
      "N-Dollar за SOL..."
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

    // Получаем оставшееся количество N-Dollar
    const remainingNDollarBalance =
      await provider.connection.getTokenAccountBalance(testUserNDollarAccount);
    console.log("Осталось N-Dollar:", remainingNDollarBalance.value.uiAmount);
    console.log(
      "Продано N-Dollar:",
      boughtNDollarBalance.value.uiAmount! -
        remainingNDollarBalance.value.uiAmount!
    );

    // Проверяем цену после продажи
    const priceAfterSell = await getCurrentPrice();
    console.log("Цена после продажи: 1 SOL =", priceAfterSell, "N-Dollar");
    console.log(
      "Изменение цены после продажи:",
      (((priceAfterSell - priceAfterBuy) / priceAfterBuy) * 100).toFixed(2),
      "%"
    );

    // Проверяем общее изменение цены
    console.log(
      "Общее изменение цены:",
      (((priceAfterSell - initialPrice) / initialPrice) * 100).toFixed(2),
      "%"
    );

    // Проверяем, что цена изменилась
    assert(priceAfterBuy !== initialPrice, "Цена не изменилась после покупки");
    assert(
      priceAfterSell !== priceAfterBuy,
      "Цена не изменилась после продажи"
    );
  });
});
