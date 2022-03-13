import { useMarket } from "@/contexts/MarketContext";
import { useSolana } from "@/contexts/SolanaContext";
import { useProvider } from "@/hooks/useProvider";
import { sendSignConfirm, unshiftCreateATA } from "@/utils";
import { getAssociatedTokenAddress } from "@solana/spl-token";
import { Transaction } from "@solana/web3.js";
import { ButtonHTMLAttributes, useState } from "react";

export function MatchButton({
  ...buttonStyles
}: ButtonHTMLAttributes<HTMLButtonElement>) {
  const { cluster } = useSolana();
  const { wallet } = useProvider();
  const { market } = useMarket();

  const [isAwaitingTx, setIsAwaitingTx] = useState(false);

  const walletPubkey = wallet.publicKey;

  const tryMatch = async () => {
    if (!walletPubkey) return;
    const highestBid = market.offers.get(market.bidOffers[0]);
    const lowestAsk = market.offers.get(market.askOffers[0]);
    if (!highestBid || !lowestAsk) {
      alert("market out of sync");
      return;
    }
    const isMatch =
      highestBid.offering * lowestAsk.offering >=
      highestBid.acceptAtLeast * lowestAsk.acceptAtLeast;
    if (!isMatch) {
      alert("no matches currently possible");
      return;
    }
    const userQuoteAcc = await getAssociatedTokenAddress(
      highestBid.offerMint,
      walletPubkey,
      true
    );
    const userBaseAcc = await getAssociatedTokenAddress(
      lowestAsk.offerMint,
      walletPubkey,
      true
    );
    const matchIx = highestBid.matchOffersInstruction(
      lowestAsk,
      userQuoteAcc,
      userBaseAcc
    );
    const tx = new Transaction();
    tx.add(matchIx);
    await unshiftCreateATA(
      market.connection,
      tx,
      highestBid.offerMint,
      walletPubkey
    );
    await unshiftCreateATA(
      market.connection,
      tx,
      lowestAsk.offerMint,
      walletPubkey
    );
    setIsAwaitingTx(true);
    await sendSignConfirm(
      cluster.network,
      market.connection,
      wallet,
      tx,
      undefined,
      "Orders matched!"
    );
    setIsAwaitingTx(false);
  };

  return (
    <button
      {...buttonStyles}
      onClick={tryMatch}
      disabled={
        !walletPubkey ||
        market.bidOffers.length === 0 ||
        market.askOffers.length === 0 ||
        isAwaitingTx
      }
    >
      {isAwaitingTx ? "Confirming..." : "Try To Match Orders"}
    </button>
  );
}
