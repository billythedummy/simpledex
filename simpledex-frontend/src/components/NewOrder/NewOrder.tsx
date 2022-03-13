import { useState, VFC } from "react";
import { Market, Side } from "simpledex";
import u from "@/styles/u.module.css";
import { TokenValueInputRow } from "./TokenValueInputRow";
import { useProvider } from "@/hooks/useProvider";
import { parseTokenVal, unshiftCreateATA } from "@/utils";
import { Transaction } from "@solana/web3.js";
import { useSolana } from "@/contexts/SolanaContext";

type NewOrderProps = {
  market: Market;
};

export const NewOrder: VFC<NewOrderProps> = ({ market }) => {
  const { cluster } = useSolana();
  const { wallet } = useProvider();

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
    try {
      const { blockhash } = await market.connection.getLatestBlockhash();
      tx.recentBlockhash = blockhash;
      tx.feePayer = walletPubkey;
      const signedTx = await wallet.signTransaction(tx);
      const rawTx = signedTx.serialize();
      const sig = await market.connection.sendRawTransaction(rawTx);
      await market.connection.confirmTransaction(sig, "confirmed");
      alert(
        `Limit order created. View on explorer: https://explorer.solana.com/tx/${sig}?cluster=${cluster.network}`
      );
    } catch (e) {
      console.log(e);
      alert(JSON.stringify(e));
    }
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
