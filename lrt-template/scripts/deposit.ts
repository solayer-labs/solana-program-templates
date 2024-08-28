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
import {
  LRT_TEMPLATE_PROGRAM_ID_DEVNET,
  STAKED_SOL_MINT_PUB_KEY_DEVNET,
  SOLAYER_SOL_MINT_PUB_KEY_DEVNET,
  SOLAYER_RESTAKE_PROGRAM_ID_DEVNET,
  SOLAYER_RESTAKE_POOL_DEVNET,
} from "./constants";
import { depositSol } from "@solana/spl-stake-pool";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";

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
    [Buffer.from("lrt_pool"), OUTPUT_TOKEN_MINT_KEYPAIR.publicKey.toBuffer()],
    program.programId
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

  const poolInputTokenBalanceBefore = await connection.getTokenAccountBalance(
    poolInputTokenVault
  );
  const userOutputTokenBalanceBefore = await connection.getTokenAccountBalance(
    signerOutputTokenVault
  );
  let tx = newTransactionWithComputeUnitPriceAndLimit();

  // airdrop sol and convert sol to staked sol through stake pool
  // however, devnet stake pool is not working at this moment
  // https://github.com/solana-labs/solana-program-library/issues/7208
  // I m intentionally skipping this step and start from staked sol directly
  // await despositSol

  // Mint staked sol directly given stake pool is not working at this moment
  const signerStakedSolVault = await createTokenAccount(
    connection,
    USER_KEYPAIR,
    STAKED_SOL_MINT_PUB_KEY_DEVNET,
    USER_KEYPAIR.publicKey
  );
  const signerStakedSolMintInst = createMintToInstruction(
    STAKED_SOL_MINT_PUB_KEY_DEVNET,
    signerStakedSolVault,
    KEYPAIR.publicKey,
    new anchor.BN(DEPOSIT_AMOUNT * LAMPORTS_PER_SOL).toNumber()
  );

  tx.add(signerStakedSolMintInst);

  // restake staked sol to get ssol
  const restakingProgramIdl = await anchor.Program.fetchIdl(
    SOLAYER_RESTAKE_PROGRAM_ID_DEVNET,
    { connection }
  );

  const restakingProgram = new Program(
    restakingProgramIdl,
    SOLAYER_RESTAKE_PROGRAM_ID_DEVNET,
    { connection }
  );

  const restakeInst = await restakingProgram.methods
    .restake(new anchor.BN(DEPOSIT_AMOUNT * LAMPORTS_PER_SOL))
    .accounts({
      signer: USER_KEYPAIR.publicKey,
      lstMint: STAKED_SOL_MINT_PUB_KEY_DEVNET,
      rstMint: SOLAYER_SOL_MINT_PUB_KEY_DEVNET,
      pool: SOLAYER_RESTAKE_POOL_DEVNET,
      vault: getAssociatedTokenAddressSync(
        STAKED_SOL_MINT_PUB_KEY_DEVNET,
        SOLAYER_RESTAKE_POOL_DEVNET,
        true
      ),
      lstAta: signerStakedSolVault,
      rstAta: getAssociatedTokenAddressSync(
        SOLAYER_SOL_MINT_PUB_KEY_DEVNET,
        USER_KEYPAIR.publicKey,
        true
      ),
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .instruction();

  tx.add(restakeInst);

  const depositInst = await program.methods
    .deposit(new anchor.BN(DEPOSIT_AMOUNT * LAMPORTS_PER_SOL))
    .accounts({
      signer: USER_KEYPAIR.publicKey,
      inputTokenMint: SOLAYER_SOL_MINT_PUB_KEY_DEVNET,
      signerInputTokenVault,
      poolInputTokenVault,
      outputTokenMint: OUTPUT_TOKEN_MINT_KEYPAIR.publicKey,
      signerOutputTokenVault,
      pool,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
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

    await new Promise((f) => setTimeout(f, 3000));

  const poolInputTokenBalanceAfter = await connection.getTokenAccountBalance(
    poolInputTokenVault
  );
  const userOutputTokenBalanceAfter = await connection.getTokenAccountBalance(
    signerOutputTokenVault
  );

  assert.equal(
    poolInputTokenBalanceAfter.value.uiAmount -
      poolInputTokenBalanceBefore.value.uiAmount,
    DEPOSIT_AMOUNT,
    "deposit amount not match"
  );
  assert.equal(
    userOutputTokenBalanceAfter.value.uiAmount -
      userOutputTokenBalanceBefore.value.uiAmount,
    DEPOSIT_AMOUNT,
    "output token amount not match"
  );
}

main().then(() => process.exit());
