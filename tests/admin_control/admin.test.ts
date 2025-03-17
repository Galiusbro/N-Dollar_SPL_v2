import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AdminControl } from "../../target/types/admin_control";
import { PublicKey, Keypair } from "@solana/web3.js";
import { expect } from "chai";
import { createMint, getMint } from "@solana/spl-token";

describe("admin_control", () => {
  // Настройка провайдера Anchor
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Программа admin_control
  const program = anchor.workspace.AdminControl as Program<AdminControl>;

  // Кошелек пользователя (админа)
  const authority = provider.wallet;

  // PDA для хранения конфигурации
  let adminConfigPda: PublicKey;
  let adminConfigBump: number;

  // Реальный mint для N-Dollar (вместо мнимого)
  let nDollarMint: PublicKey;

  // Мнимые Program ID для тестирования
  const mockBondingCurveProgram = Keypair.generate().publicKey;
  const mockGenesisProgram = Keypair.generate().publicKey;
  const mockReferralSystemProgram = Keypair.generate().publicKey;
  const mockTradingExchangeProgram = Keypair.generate().publicKey;
  const mockLiquidityManagerProgram = Keypair.generate().publicKey;

  before(async () => {
    // Получаем PDA для админской конфигурации
    [adminConfigPda, adminConfigBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("admin_config"), authority.publicKey.toBuffer()],
      program.programId
    );

    console.log("Admin Config PDA:", adminConfigPda.toString());
    console.log("Admin Config Bump:", adminConfigBump);

    // Создаем mint для N-Dollar
    nDollarMint = await createMint(
      provider.connection,
      provider.wallet.payer, // Payer
      provider.wallet.publicKey, // Mint authority
      provider.wallet.publicKey, // Freeze authority
      9 // Decimals
    );

    console.log("N-Dollar Mint created:", nDollarMint.toString());
  });

  it("Initializes admin configuration", async () => {
    // Инициализируем админскую конфигурацию
    const tx = await program.methods
      .initializeAdmin()
      .accounts({
        authority: authority.publicKey,
        adminConfig: adminConfigPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Admin initialization tx:", tx);

    // Проверяем, что конфигурация создана правильно
    const adminConfig = await program.account.adminConfig.fetch(adminConfigPda);
    expect(adminConfig.authority.toString()).to.equal(
      authority.publicKey.toString()
    );
    expect(adminConfig.bump).to.equal(adminConfigBump);
    expect(adminConfig.initializedModules).to.equal(0);
    expect(adminConfig.feeBasisPoints).to.equal(30); // 0.3% по умолчанию
  });

  it("Initializes N-Dollar module", async () => {
    // Проверяем, что mint действительно существует
    const mintInfo = await getMint(provider.connection, nDollarMint);

    console.log("N-Dollar Mint info:", mintInfo.address.toString());

    // Инициализируем модуль N-Dollar
    const tx = await program.methods
      .initializeNdollar()
      .accounts({
        authority: authority.publicKey,
        adminConfig: adminConfigPda,
        ndollarMint: nDollarMint,
      })
      .rpc();

    console.log("N-Dollar initialization tx:", tx);

    // Проверяем, что N-Dollar инициализирован правильно
    const adminConfig = await program.account.adminConfig.fetch(adminConfigPda);
    expect(adminConfig.ndollarMint.toString()).to.equal(nDollarMint.toString());
    expect(adminConfig.initializedModules).to.equal(1); // Первый бит установлен
  });

  it("Initializes Bonding Curve module", async () => {
    // Инициализируем модуль Bonding Curve
    const tx = await program.methods
      .initializeBondingCurve()
      .accounts({
        authority: authority.publicKey,
        adminConfig: adminConfigPda,
        bondingCurveProgram: mockBondingCurveProgram,
      })
      .rpc();

    console.log("Bonding Curve initialization tx:", tx);

    // Проверяем, что Bonding Curve инициализирован правильно
    const adminConfig = await program.account.adminConfig.fetch(adminConfigPda);
    expect(adminConfig.bondingCurveProgram.toString()).to.equal(
      mockBondingCurveProgram.toString()
    );

    // Биты модулей: 1 (N-Dollar) + 2 (Bonding Curve) = 3
    expect(adminConfig.initializedModules).to.equal(3); // 00000011
  });

  it("Updates fees", async () => {
    // Обновляем комиссию до 0.5%
    const newFee = 50; // 0.5%
    const tx = await program.methods
      .updateFees(newFee)
      .accounts({
        authority: authority.publicKey,
        adminConfig: adminConfigPda,
      })
      .rpc();

    console.log("Fee update tx:", tx);

    // Проверяем, что комиссия обновлена правильно
    const adminConfig = await program.account.adminConfig.fetch(adminConfigPda);
    expect(adminConfig.feeBasisPoints).to.equal(newFee);
  });

  it("Authorizes and revokes programs", async () => {
    // Тестовая программа для авторизации
    const testProgram = Keypair.generate().publicKey;

    // Авторизуем программу
    const authTx = await program.methods
      .authorizeProgram(testProgram)
      .accounts({
        authority: authority.publicKey,
        adminConfig: adminConfigPda,
      })
      .rpc();

    console.log("Program authorization tx:", authTx);

    // Проверяем, что программа авторизована
    let adminConfig = await program.account.adminConfig.fetch(adminConfigPda);
    expect(adminConfig.authorizedPrograms[0].toString()).to.equal(
      testProgram.toString()
    );

    // Отзываем авторизацию
    const revokeTx = await program.methods
      .revokeProgramAuthorization(testProgram)
      .accounts({
        authority: authority.publicKey,
        adminConfig: adminConfigPda,
      })
      .rpc();

    console.log("Program revocation tx:", revokeTx);

    // Проверяем, что авторизация отозвана
    adminConfig = await program.account.adminConfig.fetch(adminConfigPda);
    expect(adminConfig.authorizedPrograms[0].toString()).to.not.equal(
      testProgram.toString()
    );
    expect(adminConfig.authorizedPrograms[0].toString()).to.equal(
      PublicKey.default.toString()
    );
  });
});
