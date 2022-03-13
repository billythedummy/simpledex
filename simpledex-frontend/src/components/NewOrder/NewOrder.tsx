import { useState, VFC } from "react";
import { Side } from "simpledex";
import u from "@/styles/u.module.css";
import { TokenValueInputRow } from "./TokenValueInputRow";
import { useProvider } from "@/hooks/useProvider";
import { parseTokenVal, sendSignConfirm, unshiftCreateATA } from "@/utils";
import { Transaction } from "@solana/web3.js";
import { useSolana } from "@/contexts/SolanaContext";
import { useMarket } from "@/contexts/MarketContext";

export const NewOrder: VFC = () => {
  const { cluster } = useSolana();
  const { wallet } = useProvider();
  const { market } = useMarket();

  const [side, setSide] = useState<Side>("bid");
  const [baseValStr, setBaseValStr] = useState("");
  const [quoteValStr, setQuoteValStr] = useState("");
  const [isAwaitingTx, setIsAwaitingTx] = useState(false);

  const isBid = side === "bid";
  const firstText = isBid ? "I want to buy at least" : "I want to sell";
  const secondText = isBid ? "with" : "for at least";
  const buttonClass = isBid
    ? `${u["green-text"]} ${u["green-border"]}`
    : `${u["red-text"]} ${u["red-border"]}`;

  const createOrder = async () => {
    const walletPubkey = wallet.publicKey;
    if (!walletPubkey || !wallet.signTransaction) return;
    const baseToken = market.baseToken ?? (await market.loadBaseToken());
    const quoteToken = market.quoteToken ?? (await market.loadQuoteToken());
    const baseVal = parseTokenVal(baseValStr, baseToken.decimals);
    const quoteVal = parseTokenVal(quoteValStr, quoteToken.decimals);
    if (!baseVal || !quoteVal) {
      setBaseValStr("");
      setQuoteValStr("");
      return;
    }
    const [offering, acceptAtLeast] = isBid
      ? [quoteVal, baseVal]
      : [baseVal, quoteVal];
    const ix = await market.createOfferInstruction(
      walletPubkey,
      side,
      offering,
      acceptAtLeast
    );
    const tx = new Transaction();
    tx.add(ix);
    await unshiftCreateATA(
      market.connection,
      tx,
      baseToken.address,
      walletPubkey
    );
    await unshiftCreateATA(
      market.connection,
      tx,
      quoteToken.address,
      walletPubkey
    );
    setIsAwaitingTx(true);
    await sendSignConfirm(
      cluster.network,
      market.connection,
      wallet,
      tx,
      undefined,
      "Limit order created"
    );
    setIsAwaitingTx(false);
  };

  return (
    <div
      className={`${u["round-border"]} ${u["grey-bg"]} ${u["flex"]} ${u["flex-col"]} ${u["padding-20"]} ${u["children-vert-margin"]}`}
    >
      <button
        className={`${buttonClass} ${u["width-200"]}`}
        onClick={() => setSide(isBid ? "ask" : "bid")}
      >
        {isBid ? "BUY" : "SELL"}
      </button>
      <p>{firstText}</p>
      <TokenValueInputRow
        value={baseValStr}
        onChange={setBaseValStr}
        token={market.baseTokenAddr}
      />
      <p>{secondText}</p>
      <TokenValueInputRow
        value={quoteValStr}
        onChange={setQuoteValStr}
        token={market.quoteTokenAddr}
      />
      <button
        disabled={!wallet.connected || isAwaitingTx}
        className={`${u["full-width"]} ${u["background-contrast"]} ${u["no-border"]}`}
        onClick={createOrder}
      >
        {isAwaitingTx ? "CONFIRMING..." : "ORDER"}
      </button>
    </div>
  );
};
