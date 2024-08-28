import { Program } from "@coral-xyz/anchor";
import * as anchor from "@coral-xyz/anchor";
import LrtTemplate from "../target/idl/lrt_template.json";
import {
  clusterApiUrl,
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  sendAndConfirmTransaction,
  SystemProgram,
} from "@solana/web3.js";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  createMintInstructions,
  loadKeypairFromFile,
  log,
  mintSetAuthorityInstruction,
  newTransactionWithComputeUnitPriceAndLimit,
} from "./helpers";
import {
  LRT_TEMPLATE_PROGRAM_ID_DEVNET,
  SOLAYER_SOL_MINT_PUB_KEY_DEVNET,
} from "./constants";

// signer keypair
const KEYPAIR = Keypair.fromSecretKey(
  new Uint8Array([
    156, 213, 112, 118, 70, 144, 0, 183, 8, 253, 100, 218, 180, 250, 254, 252,
    85, 48, 245, 134, 56, 252, 57, 206, 164, 47, 215, 247, 219, 8, 190, 122,
    206, 132, 140, 223, 170, 178, 96, 245, 208, 40, 160, 135, 72, 255, 132, 235,
    4, 15, 35, 86, 66, 167, 108, 172, 66, 84, 186, 235, 73, 53, 211, 225,
  ])
);

// you can generate a new one for your use
const DELEGATE_AUTHORITY = loadKeypairFromFile(
  "./keys/delegate_authority.json"
);

// you can generate a new one for your use
const OUTPUT_TOKEN_MINT_KEYPAIR = loadKeypairFromFile(
  "./keys/output_token_mint.json"
);

async function main() {
  const connection = new Connection(clusterApiUrl("devnet"));
  console.log(`signer wallet public key is: ${KEYPAIR.publicKey}`);
  console.log(
    `signer wallet balance is: ${
      (await connection.getBalance(KEYPAIR.publicKey)) / LAMPORTS_PER_SOL
    } SOL`
  );

  const program = new Program(
    LrtTemplate as anchor.Idl,
    LRT_TEMPLATE_PROGRAM_ID_DEVNET,
    { connection }
  );

  const [pool, bump] = PublicKey.findProgramAddressSync(
    [Buffer.from("lrt_pool"), OUTPUT_TOKEN_MINT_KEYPAIR.publicKey.toBuffer()],
    program.programId
  );

  const poolInputTokenVault = getAssociatedTokenAddressSync(
    SOLAYER_SOL_MINT_PUB_KEY_DEVNET,
    pool,
    true
  );

  console.log("signer: ", KEYPAIR.publicKey.toBase58());
  console.log("delegate_authority: ", DELEGATE_AUTHORITY.publicKey.toBase58());
  console.log("input_token_mint: ", SOLAYER_SOL_MINT_PUB_KEY_DEVNET.toBase58());
  console.log(
    "poolInputTokenVault(init_if_needed): ",
    poolInputTokenVault.toBase58()
  );
  console.log(
    "output_token_mint: ",
    OUTPUT_TOKEN_MINT_KEYPAIR.publicKey.toBase58()
  );
  console.log("pool(init), bump: ", pool.toBase58(), bump);

  let tx = newTransactionWithComputeUnitPriceAndLimit();

  const outputTokenMintInst = await createMintInstructions(
    connection,
    KEYPAIR,
    KEYPAIR.publicKey,
    KEYPAIR.publicKey,
    9,
    OUTPUT_TOKEN_MINT_KEYPAIR
  );

  tx.add(outputTokenMintInst);

  const setAuthorityInstructions = mintSetAuthorityInstruction(
    OUTPUT_TOKEN_MINT_KEYPAIR.publicKey,
    KEYPAIR.publicKey,
    pool
  );
  tx.add(setAuthorityInstructions);

  const initializeLRTPoolInst = await program.methods
    .initialize()
    .accounts({
      signer: KEYPAIR.publicKey,
      delegateAuthority: DELEGATE_AUTHORITY.publicKey,
      inputTokenMint: SOLAYER_SOL_MINT_PUB_KEY_DEVNET,
      poolInputTokenVault,
      outputTokenMint: OUTPUT_TOKEN_MINT_KEYPAIR.publicKey,
      pool,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .instruction();
  tx.add(initializeLRTPoolInst);

  try {
    await sendAndConfirmTransaction(connection, tx, [
      KEYPAIR,
      DELEGATE_AUTHORITY,
      OUTPUT_TOKEN_MINT_KEYPAIR,
    ]).then(log);
  } catch (error) {
    console.error(error);
  }
}

main().then(() => process.exit());
