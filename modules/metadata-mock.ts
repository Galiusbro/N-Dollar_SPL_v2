import * as anchor from "@coral-xyz/anchor";
import { PublicKey, Keypair, Connection } from "@solana/web3.js";

/**
 * Мок для программы метаданных Metaplex
 * Используется для тестирования контрактов, которые используют метаданные
 */
export class MetadataProgramMock {
  private admin: Keypair;
  private connection: Connection;
  public publicKey: PublicKey;

  constructor(admin: Keypair, connection: Connection) {
    this.admin = admin;
    this.connection = connection;
    // Используем официальный ID программы метаданных Metaplex
    this.publicKey = new PublicKey(
      "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
    );
  }

  /**
   * Находит PDA для метаданных по минту
   * @param mint Адрес минта токена
   * @returns PDA для метаданных
   */
  findMetadataPDA(mint: PublicKey): PublicKey {
    const [pda] = PublicKey.findProgramAddressSync(
      [Buffer.from("metadata"), this.publicKey.toBuffer(), mint.toBuffer()],
      this.publicKey
    );
    return pda;
  }

  /**
   * В тестовом режиме мы на самом деле не создаем метаданные, это заглушка
   * В реальном окружении метаданные будут созданы через CPI вызов из программы
   */
  async mockCreateMetadata(
    metadataPDA: PublicKey,
    mint: PublicKey,
    authority: PublicKey
  ): Promise<void> {
    console.log("Mock creation of metadata account");
    console.log("Metadata PDA:", metadataPDA.toString());
    console.log("Mint:", mint.toString());
    console.log("Authority:", authority.toString());

    // В тестовом режиме мы ничего не делаем, просто эмулируем успешное создание
    // В реальном окружении программа N-Dollar Token сама создаст метаданные через CPI
    return;
  }
}
