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
import {
  ENDO_AVS_DEVNET,
  ENDO_AVS_PROGRAM_ID_DEVNET,
  ENDO_AVS_TOKEN_MINT_DEVNET,
  LRT_TEMPLATE_PROGRAM_ID_DEVNET,
  SOLAYER_SOL_MINT_PUB_KEY_DEVNET,
} from "./constants";

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
const OUTPUT_TOKEN_MINT_KEYPAIR = loadKeypairFromFile(
  "./keys/output_token_mint.json"
);

const WITHDRAW_AMOUNT = 1;

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
    [Buffer.from("lrt_pool"), OUTPUT_TOKEN_MINT_KEYPAIR.publicKey.toBuffer()],
    program.programId
  );

  const avsInputTokenVault = getAssociatedTokenAddressSync(
    SOLAYER_SOL_MINT_PUB_KEY_DEVNET,
    ENDO_AVS_DEVNET,
    true
  );

  const poolAvsTokenVault = getAssociatedTokenAddressSync(
    ENDO_AVS_TOKEN_MINT_DEVNET,
    pool,
    true
  );

  const signerInputTokenVault = getAssociatedTokenAddressSync(
    SOLAYER_SOL_MINT_PUB_KEY_DEVNET,
    USER_KEYPAIR.publicKey,
    true
  );

  const poolInputTokenVault = getAssociatedTokenAddressSync(
    SOLAYER_SOL_MINT_PUB_KEY_DEVNET,
    pool,
    true
  );

  const signerOutputTokenVault = getAssociatedTokenAddressSync(
    OUTPUT_TOKEN_MINT_KEYPAIR.publicKey,
    USER_KEYPAIR.publicKey,
    true
  );

  console.log(
    "output_token_mint: ",
    OUTPUT_TOKEN_MINT_KEYPAIR.publicKey.toBase58()
  );
  console.log("input_token_mint: ", SOLAYER_SOL_MINT_PUB_KEY_DEVNET.toBase58());
  console.log("signer_input_token_vault: ", signerInputTokenVault.toBase58());
  console.log("pool_input_token_vault: ", poolInputTokenVault.toBase58());
  console.log(
    "output_token_mint: ",
    OUTPUT_TOKEN_MINT_KEYPAIR.publicKey.toBase58()
  );
  console.log(
    "signer_output_token_vault (init_if_needed): ",
    signerOutputTokenVault.toBase58()
  );
  console.log("pool and bump: ", pool.toBase58(), bump);
  console.log("avs: ", ENDO_AVS_DEVNET.toBase58());
  console.log("AvsTokenMint:", ENDO_AVS_TOKEN_MINT_DEVNET.toBase58());
  console.log("avsInputTokenVault: ", avsInputTokenVault.toBase58());
  console.log("poolAvsTokenVault: ", poolAvsTokenVault.toBase58());

  const userOutputTokenBalanceBefore = await connection.getTokenAccountBalance(
    signerOutputTokenVault
  );
  const poolAvsTokenBalanceBefore = await connection.getTokenAccountBalance(
    poolAvsTokenVault
  );
  const userSsolBalanceBefore = await connection.getTokenAccountBalance(
    signerInputTokenVault
  );

  let tx = newTransactionWithComputeUnitPriceAndLimit();

  const withdrawInst = await program.methods
    .withdrawDelegatedStake(new anchor.BN(WITHDRAW_AMOUNT * LAMPORTS_PER_SOL))
    .accounts({
      signer: USER_KEYPAIR.publicKey,
      inputTokenMint: SOLAYER_SOL_MINT_PUB_KEY_DEVNET,
      signerInputTokenVault,
      poolInputTokenVault,
      outputTokenMint: OUTPUT_TOKEN_MINT_KEYPAIR.publicKey,
      signerOutputTokenVault,
      pool,
      avs: ENDO_AVS_DEVNET,
      avsTokenMint: ENDO_AVS_TOKEN_MINT_DEVNET,
      avsInputTokenVault,
      poolAvsTokenVault,
      avsProgram: ENDO_AVS_PROGRAM_ID_DEVNET,
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

  await new Promise((f) => setTimeout(f, 3000));

  const userOutputTokenBalanceAfter = await connection.getTokenAccountBalance(
    signerOutputTokenVault
  );
  const poolAvsTokenBalanceAfter = await connection.getTokenAccountBalance(
    poolAvsTokenVault
  );
  const userSsolBalanceAfter = await connection.getTokenAccountBalance(
    signerInputTokenVault
  );

  assert.equal(
    userOutputTokenBalanceBefore.value.uiAmount -
      userOutputTokenBalanceAfter.value.uiAmount,
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
