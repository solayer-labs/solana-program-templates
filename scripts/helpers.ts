import * as fs from "fs";
import * as anchor from "@coral-xyz/anchor";
import {
  ComputeBudgetProgram,
  type ConfirmOptions,
  Connection,
  Keypair,
  PublicKey,
  Signer,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import * as path from "path";
import {
  AuthorityType,
  createAssociatedTokenAccountIdempotent,
  createSetAuthorityInstruction,
  getMint,
} from "@solana/spl-token";
const TOKEN_PROGRAM_ID = new PublicKey(
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
);
import {
  createInitializeMint2Instruction,
  getMinimumBalanceForRentExemptMint,
  MINT_SIZE,
} from "@solana/spl-token";

export async function airdropSol(
  connection: Connection,
  publicKey: PublicKey,
  amount: number
) {
  let airdropTx = await connection.requestAirdrop(
    publicKey,
    amount * anchor.web3.LAMPORTS_PER_SOL
  );
  await confirmTransaction(connection, airdropTx);
}

export function loadKeypairFromFile(filepath: string): anchor.web3.Keypair {
  try {
    // Read the JSON keypair file
    const keypairFile = fs.readFileSync(filepath, "utf-8");
    const keypairData = JSON.parse(keypairFile);

    // Convert the keypair data to a Uint8Array
    const secretKey = Uint8Array.from(keypairData);

    // Create a Keypair object from the secret key
    const keypair = anchor.web3.Keypair.fromSecretKey(secretKey);

    return keypair;
  } catch (error) {
    console.error("Error loading keypair:", error);
    throw error;
  }
}

export async function confirmTransaction(
  connection: Connection,
  txHash: string
) {
  const latestBlockHash = await connection.getLatestBlockhash();
  await connection.confirmTransaction({
    blockhash: latestBlockHash.blockhash,
    lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
    signature: txHash,
  });
}

export async function log(signature: string): Promise<string> {
  console.log(
    `Your transaction details:
        - https://explorer.solana.com/tx/${signature}?cluster=devnet
        - https://solana.fm/tx/${signature}?cluster=devnet`
  );
  return signature;
}

export function saveKeypairToFile(
  keypair: anchor.web3.Keypair,
  filepath: string
) {
  const keypairFile = path.join(
    filepath,
    `${keypair.publicKey.toBase58()}.json`
  );
  fs.writeFileSync(keypairFile, JSON.stringify(Array.from(keypair.secretKey)));
}

export async function checkMintExistence(
  connection: Connection,
  mintAddress: PublicKey
): Promise<boolean> {
  try {
    await getMint(connection, mintAddress);
    return true;
  } catch (err) {
    return false;
  }
}

export async function createMintInstructions(
  connection: Connection,
  payer: Signer,
  mintAuthority: PublicKey,
  freezeAuthority: PublicKey | null,
  decimals: number,
  keypair = Keypair.generate(),
  confirmOptions?: ConfirmOptions,
  programId = TOKEN_PROGRAM_ID
): Promise<Transaction> {
  const lamports = await getMinimumBalanceForRentExemptMint(connection);

  const transaction = new Transaction().add(
    SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: keypair.publicKey,
      space: MINT_SIZE,
      lamports,
      programId,
    }),
    createInitializeMint2Instruction(
      keypair.publicKey,
      decimals,
      mintAuthority,
      freezeAuthority,
      programId
    )
  );

  return transaction;
}

export function newTransactionWithComputeUnitPriceAndLimit(): Transaction {
  return new Transaction().add(
    ComputeBudgetProgram.setComputeUnitLimit({
      units: 1000000,
    }),
    ComputeBudgetProgram.setComputeUnitPrice({
      microLamports: 30000,
    })
  );
}

export function mintSetAuthorityInstruction(
  account: PublicKey,
  currentAuthority: PublicKey,
  newAuthority: PublicKey | null,
  multiSigners: Signer[] = [],
  confirmOptions?: ConfirmOptions,
  programId = TOKEN_PROGRAM_ID
): Transaction {
  return new Transaction().add(
    createSetAuthorityInstruction(
      account,
      currentAuthority,
      AuthorityType.MintTokens,
      newAuthority,
      multiSigners,
      programId
    ),
    createSetAuthorityInstruction(
      account,
      currentAuthority,
      AuthorityType.FreezeAccount,
      newAuthority,
      multiSigners,
      programId
    )
  );
}

export async function createTokenAccount(
  connection: Connection,
  payer: Signer,
  mint: PublicKey,
  owner: PublicKey
) {
  return await createAssociatedTokenAccountIdempotent(
    connection,
    payer,
    mint,
    owner
  );
}
