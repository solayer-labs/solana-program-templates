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
  loadKeypairFromFile,
  log,
  newTransactionWithComputeUnitPriceAndLimit,
} from "./helpers";
import { assert } from "chai";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";

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

const SOLAYER_SOL_MINT_PUB_KEY_DEVNET = new PublicKey(
  "BQoheepVg6gprtszJFiL59pFVHPa2bu3GBZ6Un7sGGsf"
);

const LRT_TEMPLATE_PROGRAM_ID_DEVNET = new PublicKey(
  "Be419vzFciNeDWrX61Wwo2pqHWeX1JQVRQrwgoK6Lur2"
);

const LST_MINT_PUB_KEY_DEVNET = new PublicKey(
  "DaERMQKb2z7FyekFBnSYgLG9YF98AyDNVQS6VCFw8mfE"
);

const ENDO_AVS_PROGRAM_ID_DEVNET = new PublicKey(
  "DM2ReCHeTsV4fAvHsBehZBTps3DVLiK2UW2dHAYrDZrM"
);

const ENDO_AVS_DEVNET = new PublicKey(
  "GQouxK6v51z191VRdqAuudhVma7AWiqkGQ5yBWWPysqa"
);

const ENDO_AVS_TOKEN_MINT_DEVNET = new PublicKey(
  "5RA2wjzePPnk8z9Zy3whTDk4jTbMXgXqWxvCoeh8Fgck"
);

const UNELEGATE_AMOUNT = 1;

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

  const [pool, _] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("lrt_pool"),
      LST_MINT_PUB_KEY_DEVNET.toBuffer(),
      RST_MINT_KEYPAIR.publicKey.toBuffer(),
      SOLAYER_SOL_MINT_PUB_KEY_DEVNET.toBuffer(),
    ],
    program.programId
  );

  const poolDelegatedTokenAccount = getAssociatedTokenAddressSync(
    SOLAYER_SOL_MINT_PUB_KEY_DEVNET,
    pool,
    true
  );

  const poolAvsTokenAccount = getAssociatedTokenAddressSync(
    ENDO_AVS_TOKEN_MINT_DEVNET,
    pool,
    true
  );

  let tx = newTransactionWithComputeUnitPriceAndLimit();

  const poolSSolBalanceBefore = await connection.getTokenAccountBalance(
    poolDelegatedTokenAccount
  );
  const poolAvsTokenBalanceBefore = await connection.getTokenAccountBalance(
    poolAvsTokenAccount
  );

  const delegatedTokenVault = getAssociatedTokenAddressSync(
    SOLAYER_SOL_MINT_PUB_KEY_DEVNET,
    ENDO_AVS_DEVNET,
    true
  );

  const undelegateInst = await program.methods
    .undelegate(new anchor.BN(UNELEGATE_AMOUNT * LAMPORTS_PER_SOL))
    .accounts({
      signer: DELEGATE_AUTHORITY.publicKey,
      endoAvs: ENDO_AVS_DEVNET,
      avsTokenMint: ENDO_AVS_TOKEN_MINT_DEVNET,
      delegatedTokenVault,
      delegatedTokenMint: SOLAYER_SOL_MINT_PUB_KEY_DEVNET,
      poolDelegatedTokenAccount,
      poolAvsTokenAccount,
      pool,
      endoAvsProgram: ENDO_AVS_PROGRAM_ID_DEVNET,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .remainingAccounts([
      {
        pubkey: poolDelegatedTokenAccount,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: pool,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: ENDO_AVS_DEVNET,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: SOLAYER_SOL_MINT_PUB_KEY_DEVNET,
        isSigner: false,
        isWritable: true,
      },
    ])
    .instruction();
  tx.add(undelegateInst);

  try {
    await sendAndConfirmTransaction(connection, tx, [DELEGATE_AUTHORITY]).then(
      (signature: string) => {
        console.log("Undelegate Tx Success.");
        log(signature);
      }
    );
  } catch (error) {
    console.error(error);
  }

  const poolSSolBalanceAfter = await connection.getTokenAccountBalance(
    poolDelegatedTokenAccount
  );
  const poolAvsTokenBalanceAfter = await connection.getTokenAccountBalance(
    poolAvsTokenAccount
  );

  assert.equal(
    poolSSolBalanceAfter.value.uiAmount - poolSSolBalanceBefore.value.uiAmount,
    UNELEGATE_AMOUNT,
    "delegated token account balance should increase by UNELEGATE_AMOUNT"
  );

  assert.equal(
    poolAvsTokenBalanceBefore.value.uiAmount -
      poolAvsTokenBalanceAfter.value.uiAmount,
    UNELEGATE_AMOUNT,
    "avs token account balance should decrease by UNELEGATE_AMOUNT"
  );
}

main().then(() => process.exit());
