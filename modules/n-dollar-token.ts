import * as anchor from "@coral-xyz/anchor";
import { AnchorProvider, Program } from "@coral-xyz/anchor";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  Transaction,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { assert } from "chai";
import BN from "bn.js";
import { MetadataProgramMock } from "../modules/metadata-mock";

interface NDollarInitResult {
  nDollarMint: PublicKey;
  adminNDollarAccount: PublicKey;
  user1NDollarAccount: PublicKey;
  user2NDollarAccount: PublicKey;
  mockMetadataProgram: PublicKey;
}

/**
 * Инициализирует N-Dollar токен, используя смарт-контракт
 */
export async function initializeNDollar(
  provider: AnchorProvider,
  admin: Keypair,
  nDollarDecimals: number,
  name: string = "N-Dollar",
  symbol: string = "NDOL",
  uri: string = "https://example.com/ndollar.json"
): Promise<NDollarInitResult> {
  console.log("Initializing N-Dollar token using smart contract...");

  // Загружаем программу N-Dollar Token
  const nDollarProgram = anchor.workspace.NDollarToken as Program;

  // Загружаем программу Admin Control для CPI вызовов
  const adminControlProgram = anchor.workspace.AdminControl as Program;

  // Загружаем программу Liquidity Manager
  const liquidityManagerProgram = anchor.workspace.LiquidityManager as Program;

  // Создаем инстанс обработчика метаданных Metaplex
  const metadataMock = new MetadataProgramMock(admin, provider.connection);
  const metadataProgramId = metadataMock.publicKey;

  // Создаем keypair для минта N-Dollar
  const nDollarMintKeypair = Keypair.generate();
  const nDollarMint = nDollarMintKeypair.publicKey;

  // Создаем PDA для admin account
  const [adminAccountPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("admin_account"), nDollarMint.toBuffer()],
    nDollarProgram.programId
  );

  // Создаем PDA для admin_config из программы admin_control
  const [adminConfigPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("admin_config")],
    adminControlProgram.programId
  );

  // Создаем PDA для metadata с использованием нашего обработчика
  const metadataPDA = metadataMock.findMetadataPDA(nDollarMint);

  // Создаем PDA для Liquidity Manager
  const [liquidityManagerPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("liquidity_manager"), admin.publicKey.toBuffer()],
    liquidityManagerProgram.programId
  );

  // Создаем PDA для пула SOL
  const [poolSolPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("pool_sol"), liquidityManagerPDA.toBuffer()],
    liquidityManagerProgram.programId
  );

  try {
    // Предварительно создадим мок метаданных для токена
    console.log("Creating mock metadata account...");
    await metadataMock.mockCreateMetadata(
      metadataPDA,
      nDollarMint,
      admin.publicKey
    );

    // Создаем ассоциированные токен-аккаунты для админа и пользователей
    const adminNDollarAccount = await getAssociatedTokenAddress(
      nDollarMint,
      admin.publicKey
    );

    // Создаем ассоциированный токен-аккаунт для пула ликвидности
    const poolNDollarAccount = await getAssociatedTokenAddress(
      nDollarMint,
      liquidityManagerPDA,
      true // allowOwnerOffCurve для PDA
    );

    // Создаем ассоциированные токен-аккаунты в отдельной транзакции
    const createAccountsTx = new Transaction();

    // Создаем токен-аккаунт для пула ликвидности
    createAccountsTx.add(
      createAssociatedTokenAccountInstruction(
        admin.publicKey,
        poolNDollarAccount,
        liquidityManagerPDA,
        nDollarMint
      )
    );

    await anchor.web3.sendAndConfirmTransaction(
      provider.connection,
      createAccountsTx,
      [admin]
    );

    console.log("Created pool token account:", poolNDollarAccount.toString());

    // Инициализируем N-Dollar токен, используя программу Anchor
    console.log("Initializing N-Dollar token...");
    await nDollarProgram.methods
      .initializeNDollar(name, symbol, uri, nDollarDecimals)
      .accounts({
        authority: admin.publicKey,
        mint: nDollarMint,
        metadata: metadataPDA,
        adminAccount: adminAccountPDA,
        adminConfig: adminConfigPDA,
        adminControlProgram: adminControlProgram.programId,
        liquidityPoolTokenAccount: poolNDollarAccount,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        metadataProgram: metadataProgramId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([admin, nDollarMintKeypair])
      .rpc();

    console.log(
      "N-Dollar token successfully initialized with 108 million tokens in liquidity pool!"
    );

    // Инициализируем Liquidity Manager и добавляем начальную ликвидность в SOL
    console.log("Initializing Liquidity Manager...");
    await liquidityManagerProgram.methods
      .initializeLiquidityManager()
      .accounts({
        authority: admin.publicKey,
        nDollarMint: nDollarMint,
        liquidityManager: liquidityManagerPDA,
        poolNDollarAccount: poolNDollarAccount,
        poolSolAccount: poolSolPDA,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([admin])
      .rpc();

    console.log("Liquidity Manager successfully initialized!");

    // Проверяем баланс пула
    const poolBalance = await provider.connection.getTokenAccountBalance(
      poolNDollarAccount
    );
    console.log("Pool N-Dollar balance:", poolBalance.value.uiAmount, "NDOL");

    // Для тестирования создаем пару пользователей
    const user1 = Keypair.generate();
    const user2 = Keypair.generate();

    // Выдаем им SOL для транзакций
    await provider.connection.requestAirdrop(
      user1.publicKey,
      anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.requestAirdrop(
      user2.publicKey,
      anchor.web3.LAMPORTS_PER_SOL
    );

    // Создаем ассоциированные токен-аккаунты
    const user1NDollarAccount = await getAssociatedTokenAddress(
      nDollarMint,
      user1.publicKey
    );

    const user2NDollarAccount = await getAssociatedTokenAddress(
      nDollarMint,
      user2.publicKey
    );

    // Создаем ассоциированные токен-аккаунты для пользователей
    const userAccountsTx = new Transaction();

    userAccountsTx.add(
      createAssociatedTokenAccountInstruction(
        admin.publicKey,
        adminNDollarAccount,
        admin.publicKey,
        nDollarMint
      )
    );

    userAccountsTx.add(
      createAssociatedTokenAccountInstruction(
        admin.publicKey,
        user1NDollarAccount,
        user1.publicKey,
        nDollarMint
      )
    );

    userAccountsTx.add(
      createAssociatedTokenAccountInstruction(
        admin.publicKey,
        user2NDollarAccount,
        user2.publicKey,
        nDollarMint
      )
    );

    await anchor.web3.sendAndConfirmTransaction(
      provider.connection,
      userAccountsTx,
      [admin]
    );

    console.log("Created user token accounts:");
    console.log("Admin token account:", adminNDollarAccount.toString());
    console.log("User1 token account:", user1NDollarAccount.toString());
    console.log("User2 token account:", user2NDollarAccount.toString());

    return {
      nDollarMint,
      adminNDollarAccount,
      user1NDollarAccount,
      user2NDollarAccount,
      mockMetadataProgram: metadataProgramId,
    };
  } catch (error) {
    console.error("Error initializing N-Dollar:", error);
    throw error;
  }
}
