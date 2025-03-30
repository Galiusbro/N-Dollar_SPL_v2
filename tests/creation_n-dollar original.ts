// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import { PublicKey, Keypair } from "@solana/web3.js";
// import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

// describe("N-Dollar Token Creation Test", () => {
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//   const program = anchor.workspace.NDollar as Program;
//   const wallet = provider.wallet as anchor.Wallet;
//   const METADATA_PROGRAM_ID = new PublicKey(
//     "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
//   );

//   // Объявляем переменные без начальных значений
//   let mint: Keypair;
//   let metadataPda: PublicKey;

//   // Добавляем функцию для аирдропа
//   async function airdropSol(address: PublicKey, amount: number) {
//     const signature = await provider.connection.requestAirdrop(address, amount);
//     await provider.connection.confirmTransaction(signature);
//     console.log(
//       `Airdropped ${
//         amount / anchor.web3.LAMPORTS_PER_SOL
//       } SOL to ${address.toString()}`
//     );
//   }

//   // Функция для чтения строки из буфера
//   function readMetaplexString(
//     buffer: Buffer,
//     offset: number
//   ): [string, number] {
//     // В формате Metaplex первые 4 байта - длина строки
//     const length = buffer.readUInt32LE(offset);
//     console.log(`Reading string at offset ${offset}, length: ${length}`);
//     if (length === 0) {
//       return ["", offset + 4];
//     }
//     const str = buffer.slice(offset + 4, offset + 4 + length).toString("utf8");
//     console.log(`Read string: "${str}"`);
//     return [str, offset + 4 + length];
//   }

//   before(async () => {
//     // Инициализируем все PDA и аккаунты перед тестами
//     mint = Keypair.generate();

//     [metadataPda] = PublicKey.findProgramAddressSync(
//       [
//         Buffer.from("metadata"),
//         METADATA_PROGRAM_ID.toBuffer(),
//         mint.publicKey.toBuffer(),
//       ],
//       METADATA_PROGRAM_ID
//     );

//     // Добавляем аирдроп для пользователя
//     const userBalance = await provider.connection.getBalance(wallet.publicKey);
//     if (userBalance < anchor.web3.LAMPORTS_PER_SOL) {
//       await airdropSol(wallet.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL); // 2 SOL для тестов
//     }
//     console.log(
//       "User SOL balance:",
//       (await provider.connection.getBalance(wallet.publicKey)) /
//         anchor.web3.LAMPORTS_PER_SOL,
//       "SOL"
//     );
//   });

//   it("Creates token with metadata", async () => {
//     const tx = await program.methods
//       .createToken("One-Click Token", "ONE", "https://oneclick.com/token.json")
//       .accounts({
//         mint: mint.publicKey,
//         metadata: metadataPda,
//         authority: wallet.publicKey,
//         tokenProgram: TOKEN_PROGRAM_ID,
//         systemProgram: anchor.web3.SystemProgram.programId,
//         rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//         tokenMetadataProgram: METADATA_PROGRAM_ID,
//       })
//       .signers([mint])
//       .rpc();

//     console.log("Create token tx:", tx);

//     // Проверяем, что минт создан
//     const mintInfo = await provider.connection.getAccountInfo(mint.publicKey);
//     console.log("Mint created:", mintInfo !== null);

//     // Проверяем, что метаданные созданы и выводим их
//     const metadataInfo = await provider.connection.getAccountInfo(metadataPda);
//     if (metadataInfo) {
//       try {
//         // Пропускаем первые 1 + 32 + 32 байта (версия + update authority + mint)
//         let offset = 1 + 32 + 32;

//         // Читаем имя
//         const [name, nameEnd] = readMetaplexString(metadataInfo.data, offset);

//         // Читаем символ
//         const [symbol, symbolEnd] = readMetaplexString(
//           metadataInfo.data,
//           nameEnd
//         );

//         // Читаем URI
//         const [uri] = readMetaplexString(metadataInfo.data, symbolEnd);

//         console.log("\nToken Metadata:");
//         console.log("Name:", name);
//         console.log("Symbol:", symbol);
//         console.log("URI:", uri);
//       } catch (error) {
//         console.log("Error reading metadata:", error);
//         console.log(
//           "Error details:",
//           error instanceof Error ? error.message : error
//         );
//       }
//     } else {
//       console.log("Metadata account not found");
//     }
//   });
// });
