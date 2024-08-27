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
  createMintToInstruction,
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  airdropSol,
  createTokenAccount,
  loadKeypairFromFile,
  log,
  newTransactionWithComputeUnitPriceAndLimit,
} from "./helpers";
import { assert } from "chai";

// LST mint admin keypair
const KEYPAIR = Keypair.fromSecretKey(
  new Uint8Array([
    156, 213, 112, 118, 70, 144, 0, 183, 8, 253, 100, 218, 180, 250, 254, 252,
    85, 48, 245, 134, 56, 252, 57, 206, 164, 47, 215, 247, 219, 8, 190, 122,
    206, 132, 140, 223, 170, 178, 96, 245, 208, 40, 160, 135, 72, 255, 132, 235,
    4, 15, 35, 86, 66, 167, 108, 172, 66, 84, 186, 235, 73, 53, 211, 225,
  ])
);

const USER_KEYPAIR = loadKeypairFromFile("./keys/user.json");

// use the same one as initialize
const RST_MINT_KEYPAIR = loadKeypairFromFile("./keys/rst_mint.json");

const LRT_TEMPLATE_PROGRAM_ID_DEVNET = new PublicKey(
  "Be419vzFciNeDWrX61Wwo2pqHWeX1JQVRQrwgoK6Lur2"
);

const LST_MINT_PUB_KEY_DEVNET = new PublicKey(
  "DaERMQKb2z7FyekFBnSYgLG9YF98AyDNVQS6VCFw8mfE"
);

const SOLAYER_SOL_MINT_PUB_KEY_DEVNET = new PublicKey(
  "BQoheepVg6gprtszJFiL59pFVHPa2bu3GBZ6Un7sGGsf"
);

const SOLAYER_RESTAKE_POOL_DEVNET = new PublicKey(
  "HukzvthPRkQYYon61o1ZKmwU4pxVL8ahMzTzsmWcEB5F"
);

const SOLAYER_RESTAKE_PROGRAM_ID_DEVNET = new PublicKey(
  "3uZbsFKoxpX8NaRWgkMRebVCofCWoTcJ3whrt4Lvoqn9"
);

const DEPOSIT_AMOUNT = 10;

async function main() {
  const connection = new Connection(clusterApiUrl("devnet"));
  if ((await connection.getBalance(USER_KEYPAIR.publicKey)) < 0.2) {
    await airdropSol(connection, USER_KEYPAIR.publicKey, 2);
  }
  console.log(`signer wallet public key is: ${USER_KEYPAIR.publicKey}`);
  console.log(
    `signer wallet balance is: ${
      (await connection.getBalance(USER_KEYPAIR.publicKey)) / LAMPORTS_PER_SOL
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

  const poolLstVault = getAssociatedTokenAddressSync(
    LST_MINT_PUB_KEY_DEVNET,
    pool,
    true
  );

  const lstAta = await createTokenAccount(
    connection,
    USER_KEYPAIR,
    LST_MINT_PUB_KEY_DEVNET,
    USER_KEYPAIR.publicKey
  );

  const rstAta = getAssociatedTokenAddressSync(
    RST_MINT_KEYPAIR.publicKey,
    USER_KEYPAIR.publicKey,
    true
  );

  const poolSsolAta = getAssociatedTokenAddressSync(
    SOLAYER_SOL_MINT_PUB_KEY_DEVNET,
    pool,
    true
  );

  const restakingPoolLstVault = getAssociatedTokenAddressSync(
    LST_MINT_PUB_KEY_DEVNET,
    SOLAYER_RESTAKE_POOL_DEVNET,
    true
  );

  console.log(
    "poolLstVault(init_if_needed during initialization): ",
    poolLstVault.toBase58()
  );
  console.log("pool and bump: ", pool.toBase58(), bump);
  console.log("rst_mint: ", RST_MINT_KEYPAIR.publicKey.toBase58());
  console.log("lstAta: ", lstAta.toBase58());
  console.log("rstAta(init_if_needed): ", rstAta.toBase58());
  console.log(
    "poolSsolAta(init_if_needed during initialization): ",
    poolSsolAta.toBase58()
  );
  console.log(
    "restakingPoolLstVault(inited by restaking program): ",
    restakingPoolLstVault.toBase58()
  );

  const poolSSolBalanceBefore = await connection.getTokenAccountBalance(
    poolSsolAta
  );

  let tx = newTransactionWithComputeUnitPriceAndLimit();

  const lstAtaMintInst = createMintToInstruction(
    LST_MINT_PUB_KEY_DEVNET,
    lstAta,
    KEYPAIR.publicKey,
    new anchor.BN(DEPOSIT_AMOUNT * LAMPORTS_PER_SOL).toNumber()
  );

  tx.add(lstAtaMintInst);

  const depositInst = await program.methods
    .deposit(new anchor.BN(DEPOSIT_AMOUNT * LAMPORTS_PER_SOL))
    .accounts({
      signer: USER_KEYPAIR.publicKey,
      lstMint: LST_MINT_PUB_KEY_DEVNET,
      lstAta,
      rstMint: RST_MINT_KEYPAIR.publicKey,
      rstAta,
      vault: poolLstVault,
      ssolMint: SOLAYER_SOL_MINT_PUB_KEY_DEVNET,
      ssolAta: poolSsolAta,
      restakingPoolLstVault,
      pool,
      restakingPool: SOLAYER_RESTAKE_POOL_DEVNET,
      restakingProgram: SOLAYER_RESTAKE_PROGRAM_ID_DEVNET,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .remainingAccounts([
      {
        pubkey: pool,
        isSigner: false,
        isWritable: true,
      },
    ])
    .instruction();
  tx.add(depositInst);

  await sendAndConfirmTransaction(connection, tx, [KEYPAIR, USER_KEYPAIR])
    .then((signature: string) => {
      console.log("Deposit Tx Success.");
      log(signature);
    })
    .catch((e) => {
      console.error(e);
    });

  const poolSSolBalanceAfter = await connection.getTokenAccountBalance(
    poolSsolAta
  );

  assert.equal(
    poolSSolBalanceAfter.value.uiAmount - poolSSolBalanceBefore.value.uiAmount,
    DEPOSIT_AMOUNT,
    "deposit amount not match"
  );
}

main().then(() => process.exit());
