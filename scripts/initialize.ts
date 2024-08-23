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
const RST_MINT_KEYPAIR = loadKeypairFromFile("./keys/rst_mint.json");

const SOLAYER_RESTAKE_PROGRAM_ID_DEVNET = new PublicKey(
  "3uZbsFKoxpX8NaRWgkMRebVCofCWoTcJ3whrt4Lvoqn9"
);

const SOLAYER_SOL_MINT_PUB_KEY_DEVNET = new PublicKey(
  "BQoheepVg6gprtszJFiL59pFVHPa2bu3GBZ6Un7sGGsf"
);

const LRT_TEMPLATE_PROGRAM_ID_DEVNET = new PublicKey(
  "Be419vzFciNeDWrX61Wwo2pqHWeX1JQVRQrwgoK6Lur2"
);

const LST_MINT_PUB_KEY_DEVNET = new PublicKey(
  "DaERMQKb2z7FyekFBnSYgLG9YF98AyDNVQS6VCFw8mfE"
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
    [
      Buffer.from("lrt_pool"),
      LST_MINT_PUB_KEY_DEVNET.toBuffer(),
      RST_MINT_KEYPAIR.publicKey.toBuffer(),
      SOLAYER_SOL_MINT_PUB_KEY_DEVNET.toBuffer(),
    ],
    program.programId
  );

  const lstVault = getAssociatedTokenAddressSync(
    LST_MINT_PUB_KEY_DEVNET,
    pool,
    true
  );

  const ssolVault = getAssociatedTokenAddressSync(
    SOLAYER_SOL_MINT_PUB_KEY_DEVNET,
    pool,
    true
  );

  console.log("lst_mint: ", LST_MINT_PUB_KEY_DEVNET.toBase58());
  console.log("lst_vault(init_if_needed): ", lstVault.toBase58());
  console.log("pool(init), bump: ", pool.toBase58(), bump);
  console.log("rst_mint: ", RST_MINT_KEYPAIR.publicKey.toBase58());
  console.log("delegate_authority: ", DELEGATE_AUTHORITY.publicKey.toBase58());
  console.log("solayer_sol_mint: ", SOLAYER_SOL_MINT_PUB_KEY_DEVNET.toBase58());
  console.log("solayer_sol_vault(init_if_needed): ", ssolVault.toBase58());

  let tx = newTransactionWithComputeUnitPriceAndLimit();

  const rstMintInst = await createMintInstructions(
    connection,
    KEYPAIR,
    KEYPAIR.publicKey,
    KEYPAIR.publicKey,
    9,
    RST_MINT_KEYPAIR
  );

  tx.add(rstMintInst);

  const setAuthorityInstructions = mintSetAuthorityInstruction(
    RST_MINT_KEYPAIR.publicKey,
    KEYPAIR.publicKey,
    pool
  );
  tx.add(setAuthorityInstructions);

  const initializeLRTPoolInst = await program.methods
    .initialize()
    .accounts({
      signer: KEYPAIR.publicKey,
      delegateAuthority: DELEGATE_AUTHORITY.publicKey,
      lstMint: LST_MINT_PUB_KEY_DEVNET,
      lstVault,
      rstMint: RST_MINT_KEYPAIR.publicKey,
      ssolMint: SOLAYER_SOL_MINT_PUB_KEY_DEVNET,
      ssolVault,
      pool,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .instruction();
  tx.add(initializeLRTPoolInst);

  await sendAndConfirmTransaction(connection, tx, [
    KEYPAIR,
    DELEGATE_AUTHORITY,
    RST_MINT_KEYPAIR,
  ]).then(log);
}

main().then(() => process.exit());
