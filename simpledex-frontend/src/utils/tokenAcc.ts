import {
  createAssociatedTokenAccountInstruction,
  getAccount,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import { Connection, PublicKey, Transaction } from "@solana/web3.js";

/// Check if the ATA for a token exists, and add the instruction for creating it to `tx` if it does not
export async function unshiftCreateATA(
  connection: Connection,
  tx: Transaction,
  token: PublicKey,
  user: PublicKey
): Promise<void> {
  const userATA = await getAssociatedTokenAddress(token, user, true);
  try {
    await getAccount(connection, userATA);
  } catch (e) {
    // userTargetAccount doesnt exist, create it
    console.log(
      `User ATA for target mint ${token.toString()} does not exist, creating...`
    );
    tx.instructions.unshift(
      createAssociatedTokenAccountInstruction(user, userATA, user, token)
    );
  }
}
