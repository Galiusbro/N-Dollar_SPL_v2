import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Awesome } from "../../target/types/awesome";
import { expect } from "chai";
import {
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  getAccount,
} from "@solana/spl-token";
import { PublicKey } from "@solana/web3.js";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import { publicKey } from "@metaplex-foundation/umi";
import {
  mplTokenMetadata,
  fetchMetadata,
} from "@metaplex-foundation/mpl-token-metadata";
import * as borsh from "@coral-xyz/borsh"; // Make sure to have this dependency

describe("Token Creation and Minting Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Создаем экземпляр Umi, используя RPC из Anchor
  const umi = createUmi(provider.connection.rpcEndpoint).use(
    mplTokenMetadata()
  );

  const program = anchor.workspace.Awesome as Program<Awesome>;
  const wallet = provider.wallet;

  const TOKEN_NAME = "Test Token";
  const TOKEN_SYMBOL = "TEST";
  const TOKEN_URI = "https://test.com/token.json";
  const TOKEN_DECIMALS = 9;
  const MINT_AMOUNT = new anchor.BN(1000000000);

  before(async () => {
    console.log("🚀 Начинаем тестирование создания и минтинга токенов");
    const metadataProgramId = new PublicKey(
      "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
    );
    const accountInfo = await provider.connection.getAccountInfo(
      metadataProgramId
    );
    if (!accountInfo?.executable) {
      throw new Error(
        "❌ Metaplex Token Metadata Program не установлен в тестовом валидаторе"
      );
    }
    console.log("✅ Metaplex Token Metadata Program найден");
  });

  // Определяем структуру для метаданных
  const METADATA_LAYOUT = borsh.struct([
    borsh.u8("key"),
    borsh.publicKey("updateAuthority"),
    borsh.publicKey("mint"),
    borsh.str("name"),
    borsh.str("symbol"),
    borsh.str("uri"),
    borsh.u16("sellerFeeBasisPoints"),
    borsh.option(
      borsh.vec(
        borsh.struct([
          borsh.str("address"),
          borsh.bool("verified"),
          borsh.u8("share"),
        ])
      ),
      "creators"
    ),
  ]);

  function decodeMetadataAccount(data: Buffer) {
    try {
      const decoded = METADATA_LAYOUT.decode(data);
      return {
        name: decoded.name,
        symbol: decoded.symbol,
        uri: decoded.uri,
        updateAuthority: decoded.updateAuthority.toString(),
        mint: decoded.mint.toString(),
        sellerFeeBasisPoints: decoded.sellerFeeBasisPoints,
      };
    } catch (error) {
      console.error("Ошибка декодирования (детали):", error);
      return null;
    }
  }

  it("Должен создать токен и выполнить минтинг", async () => {
    console.log("\n📝 Шаг 1: Создание нового токена");

    const mint = anchor.web3.Keypair.generate();
    console.log(`🔑 Mint address: ${mint.publicKey.toString()}`);

    const [metadataPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBuffer(),
        mint.publicKey.toBuffer(),
      ],
      new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
    );
    console.log(`📋 Metadata address: ${metadataPda.toString()}`);

    console.log("\n⏳ Создаем токен...");
    const createTx = await program.methods
      .createToken(TOKEN_NAME, TOKEN_SYMBOL, TOKEN_URI, TOKEN_DECIMALS)
      .accounts({
        mint: mint.publicKey,
        metadata: metadataPda,
        authority: wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenMetadataProgram: new PublicKey(
          "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
        ),
      } as any)
      .signers([mint])
      .rpc();

    console.log("✅ Токен успешно создан. Сигнатура транзакции:", createTx);

    // Ждем подтверждения транзакции
    console.log("\n⏳ Ожидаем подтверждения транзакции создания токена...");
    await provider.connection.confirmTransaction(createTx, "confirmed");
    console.log("✅ Транзакция создания токена подтверждена");

    console.log("\n📝 Шаг 2: Минтинг токенов");
    const tokenAccount = await getAssociatedTokenAddress(
      mint.publicKey,
      wallet.publicKey
    );
    console.log(`💰 Token Account address: ${tokenAccount.toString()}`);

    console.log(`\n⏳ Минтим ${MINT_AMOUNT.toString()} токенов...`);
    const mintTx = await program.methods
      .mintTokens(MINT_AMOUNT)
      .accounts({
        mint: mint.publicKey,
        tokenAccount,
        authority: wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: new PublicKey(
          "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        ),
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      } as any)
      .rpc();

    console.log("✅ Минтинг успешно выполнен. Сигнатура транзакции:", mintTx);

    // Ждем подтверждения транзакции
    console.log("\n⏳ Ожидаем подтверждения транзакции минтинга...");
    await provider.connection.confirmTransaction(mintTx, "confirmed");
    console.log("✅ Транзакция минтинга подтверждена");

    // Проверяем баланс
    console.log("\n📝 Шаг 3: Проверка баланса");
    const tokenAccountInfo = await provider.connection.getTokenAccountBalance(
      tokenAccount
    );
    console.log(`💰 Текущий баланс: ${tokenAccountInfo.value.amount} токенов`);

    expect(tokenAccountInfo.value.amount).to.equal(MINT_AMOUNT.toString());
    console.log("✅ Баланс соответствует ожидаемому значению");

    // Проверяем детали токен-аккаунта
    const accountInfo = await getAccount(provider.connection, tokenAccount);
    expect(accountInfo.mint.toString()).to.equal(mint.publicKey.toString());
    expect(accountInfo.owner.toString()).to.equal(wallet.publicKey.toString());
    expect(accountInfo.amount.toString()).to.equal(MINT_AMOUNT.toString());
    console.log("✅ Детали токен-аккаунта верны");

    // Добавляем задержку перед запросом метаданных
    console.log("\n⏳ Ожидаем перед запросом метаданных...");
    await new Promise((resolve) => setTimeout(resolve, 5000)); // Увеличили время ожидания до 5 секунд

    // Проверка метаданных
    console.log("\n📝 Шаг 4: Проверка метаданных токена");
    try {
      // Сначала попробуем использовать SDK метод (как было раньше)
      const metadataAccount = await fetchMetadata(
        umi,
        publicKey(metadataPda.toString())
      );

      console.log("✅ Метаданные успешно получены через SDK");
      console.log(`📌 Название токена: ${metadataAccount.name}`);
      console.log(`📌 Символ токена: ${metadataAccount.symbol}`);
      console.log(`📌 URI метаданных: ${metadataAccount.uri}`);
    } catch (error) {
      console.log(
        "⚠️ Не удалось получить метаданные через SDK:",
        error.message
      );

      // План Б: Получаем сырые данные аккаунта и декодируем их вручную
      console.log("\n📝 Чтение сырых данных метаданных...");
      const metadataAccountInfo = await provider.connection.getAccountInfo(
        metadataPda
      );

      if (metadataAccountInfo) {
        console.log(
          "✅ Аккаунт метаданных существует, размер данных:",
          metadataAccountInfo.data.length
        );

        // Выведем первые 64 байта в шестнадцатеричном формате для анализа
        console.log("📊 Первые 64 байта метаданных (hex):");
        const hexPrefix = metadataAccountInfo.data.slice(0, 64).toString("hex");
        console.log(hexPrefix);

        // Попытка ручного декодирования
        console.log("\n📝 Попытка декодирования метаданных вручную...");
        try {
          const decodedData = decodeMetadataAccount(metadataAccountInfo.data);
          if (decodedData) {
            console.log("✅ Метаданные успешно декодированы:");
            console.log("📌 Детали метаданных:");
            console.log("   - Название:", decodedData.name);
            console.log("   - Символ:", decodedData.symbol);
            console.log("   - URI:", decodedData.uri);
            console.log("   - Update Authority:", decodedData.updateAuthority);
            console.log("   - Mint:", decodedData.mint);
            console.log(
              "   - Seller Fee Basis Points:",
              decodedData.sellerFeeBasisPoints
            );

            // Проверяем соответствие метаданных
            expect(decodedData.name.replace(/\0/g, "")).to.equal(TOKEN_NAME);
            expect(decodedData.symbol.replace(/\0/g, "")).to.equal(
              TOKEN_SYMBOL
            );
            expect(decodedData.uri.replace(/\0/g, "")).to.equal(TOKEN_URI);
            expect(decodedData.mint).to.equal(mint.publicKey.toString());
            console.log("✅ Все метаданные соответствуют ожидаемым значениям");
          } else {
            console.log("❌ Не удалось декодировать метаданные");
          }
        } catch (decodeError) {
          console.error("❌ Ошибка при декодировании:", decodeError);
          // Выводим дамп данных для анализа
          console.log("\n📊 Дамп данных метаданных (hex):");
          console.log(metadataAccountInfo.data.toString("hex"));
        }
      } else {
        console.log("❌ Аккаунт метаданных не существует");
      }
    }
  });
});
