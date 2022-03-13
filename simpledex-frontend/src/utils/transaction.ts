import { WalletContextState } from "@solana/wallet-adapter-react";
import { Cluster, Connection, Signer, Transaction } from "@solana/web3.js";

export async function sendSignConfirm(
  cluster: Cluster,
  connection: Connection,
  wallet: WalletContextState,
  tx: Transaction,
  additionalSigners?: Signer[],
  customMsg?: string
): Promise<void> {
  try {
    const walletPubkey = wallet.publicKey;
    if (!walletPubkey || !wallet.signTransaction) {
      throw new Error("wallet can't sign tx");
    }
    const { blockhash } = await connection.getLatestBlockhash();
    tx.recentBlockhash = blockhash;
    tx.feePayer = walletPubkey;
    if (additionalSigners) {
      tx.partialSign(...additionalSigners);
    }
    const signedTx = await wallet.signTransaction(tx);
    const rawTx = signedTx.serialize();
    const sig = await connection.sendRawTransaction(rawTx);
    await connection.confirmTransaction(sig, "confirmed");
    const success = `${
      customMsg ? `${customMsg}. ` : ""
    }View on explorer: https://explorer.solana.com/tx/${sig}?cluster=${cluster}`;
    alert(success);
  } catch (e) {
    console.log(e);
    alert(JSON.stringify(e));
  }
}
