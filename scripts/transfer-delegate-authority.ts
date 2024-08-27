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
} from "@solana/web3.js";
import {
  loadKeypairFromFile,
  log,
  newTransactionWithComputeUnitPriceAndLimit,
} from "./helpers";
import { assert } from "chai";

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

const NEW_DELEGATE_AUTHORITY = loadKeypairFromFile(
  "./keys/new_delegate_authority.json"
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

  let tx1 = newTransactionWithComputeUnitPriceAndLimit();

  const poolAccountInfoBefore = await program.account.lrtPool.fetch(pool);
  assert.equal(
    DELEGATE_AUTHORITY.publicKey.toBase58(),
    (poolAccountInfoBefore.delegateAuthority as PublicKey).toBase58()
  );

  const transferDelegateAuthorityInst = await program.methods
    .transferDelegateAuthority()
    .accounts({
      authority: DELEGATE_AUTHORITY.publicKey,
      pool,
      newAuthority: NEW_DELEGATE_AUTHORITY.publicKey,
    })
    .instruction();
  tx1.add(transferDelegateAuthorityInst);

  await sendAndConfirmTransaction(connection, tx1, [DELEGATE_AUTHORITY]);

  setTimeout(() => {}, 3000);

  const poolAccountInfoAfter = await program.account.lrtPool.fetch(pool);
  assert.equal(
    NEW_DELEGATE_AUTHORITY.publicKey.toBase58(),
    (poolAccountInfoAfter.delegateAuthority as PublicKey).toBase58()
  );

  let tx2 = newTransactionWithComputeUnitPriceAndLimit();
  const transferDelegateAuthorityBackInst = await program.methods
    .transferDelegateAuthority()
    .accounts({
      authority: NEW_DELEGATE_AUTHORITY.publicKey,
      pool,
      newAuthority: DELEGATE_AUTHORITY.publicKey,
    })
    .instruction();
  tx2.add(transferDelegateAuthorityBackInst);

  await sendAndConfirmTransaction(connection, tx2, [
    NEW_DELEGATE_AUTHORITY,
  ]).then((signature: string) => {
    console.log("Transfer Delegate Authority Tx Success.");
    log(signature);
  });

  setTimeout(() => {}, 3000);

  const poolAccountInfoRevert = await program.account.lrtPool.fetch(pool);
  assert.equal(
    DELEGATE_AUTHORITY.publicKey.toBase58(),
    (poolAccountInfoRevert.delegateAuthority as PublicKey).toBase58()
  );
}

main().then(() => process.exit());
