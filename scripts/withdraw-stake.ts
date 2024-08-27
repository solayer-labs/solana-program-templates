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
  airdropSol,
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

const ENDO_AVS_DEVNET = new PublicKey(
  "GQouxK6v51z191VRdqAuudhVma7AWiqkGQ5yBWWPysqa"
);

const ENDO_AVS_TOKEN_MINT_DEVNET = new PublicKey(
  "5RA2wjzePPnk8z9Zy3whTDk4jTbMXgXqWxvCoeh8Fgck"
);

const ENDO_AVS_PROGRAM_ID_DEVNET = new PublicKey(
  "DM2ReCHeTsV4fAvHsBehZBTps3DVLiK2UW2dHAYrDZrM"
);

const WITHDRAW_AMOUNT = 10;

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

  const signerSsolAta = getAssociatedTokenAddressSync(
    SOLAYER_SOL_MINT_PUB_KEY_DEVNET,
    USER_KEYPAIR.publicKey,
    true
  );

  const delegatedTokenVault = getAssociatedTokenAddressSync(
    SOLAYER_SOL_MINT_PUB_KEY_DEVNET,
    ENDO_AVS_DEVNET,
    true
  );

  const poolAvsTokenAccount = getAssociatedTokenAddressSync(
    ENDO_AVS_TOKEN_MINT_DEVNET,
    pool,
    true
  );

  console.log("rst_mint: ", RST_MINT_KEYPAIR.publicKey.toBase58());
  console.log("rstAta: ", rstAta.toBase58());
  console.log("SsolMint: ", SOLAYER_SOL_MINT_PUB_KEY_DEVNET.toBase58());
  console.log("SsolAta: ", poolSsolAta.toBase58());
  console.log("SignerSsolAta: ", signerSsolAta.toBase58());
  console.log("pool and bump: ", pool.toBase58(), bump);
  console.log("EndoAvs: ", ENDO_AVS_DEVNET.toBase58());
  console.log("AvsTokenMint:", ENDO_AVS_TOKEN_MINT_DEVNET.toBase58());
  console.log("DelegatedTokenVault: ", delegatedTokenVault.toBase58());
  console.log("PoolAvsTokenAccount: ", poolAvsTokenAccount.toBase58());

  const userRstBalanceBefore = await connection.getTokenAccountBalance(rstAta);
  const poolAvsTokenBalanceBefore = await connection.getTokenAccountBalance(
    poolAvsTokenAccount
  );
  const userSsolBalanceBefore = await connection.getTokenAccountBalance(
    signerSsolAta
  );

  let tx = newTransactionWithComputeUnitPriceAndLimit();

  const withdrawInst = await program.methods
    .withdrawDelegatedStake(new anchor.BN(WITHDRAW_AMOUNT * LAMPORTS_PER_SOL))
    .accounts({
      signer: USER_KEYPAIR.publicKey,
      rstMint: RST_MINT_KEYPAIR.publicKey,
      rstAta,
      ssolMint: SOLAYER_SOL_MINT_PUB_KEY_DEVNET,
      ssolAta: poolSsolAta,
      signerSsolAta,
      pool,
      endoAvs: ENDO_AVS_DEVNET,
      avsTokenMint: ENDO_AVS_TOKEN_MINT_DEVNET,
      delegatedTokenVault,
      poolAvsTokenAccount,
      endoAvsProgram: ENDO_AVS_PROGRAM_ID_DEVNET,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .remainingAccounts([
      {
        pubkey: ENDO_AVS_DEVNET,
        isSigner: false,
        isWritable: true,
      },
    ])
    .instruction();
  tx.add(withdrawInst);

  await sendAndConfirmTransaction(connection, tx, [KEYPAIR, USER_KEYPAIR])
    .then((signature: string) => {
      console.log("Withdraw Stake Tx Success.");
      log(signature);
    })
    .catch((e) => {
      console.error(e);
    });

  setTimeout(() => {}, 3000);

  const userRstBalanceAfter = await connection.getTokenAccountBalance(rstAta);
  const poolAvsTokenBalanceAfter = await connection.getTokenAccountBalance(
    poolAvsTokenAccount
  );
  const userSsolBalanceAfter = await connection.getTokenAccountBalance(
    signerSsolAta
  );

  assert.equal(
    userRstBalanceBefore.value.uiAmount - userRstBalanceAfter.value.uiAmount,
    WITHDRAW_AMOUNT,
    "withdraw stake amount not match"
  );

  assert.equal(
    poolAvsTokenBalanceBefore.value.uiAmount -
      poolAvsTokenBalanceAfter.value.uiAmount,
    WITHDRAW_AMOUNT,
    "pool avs token account balance should decrease by WITHDRAW_AMOUNT"
  );

  assert.equal(
    userSsolBalanceAfter.value.uiAmount - userSsolBalanceBefore.value.uiAmount,
    WITHDRAW_AMOUNT,
    "user ssol balance should increase by WITHDRAW_AMOUNT"
  );
}

main().then(() => process.exit());
