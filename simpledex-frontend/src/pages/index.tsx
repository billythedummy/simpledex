import { PublicKey } from "@solana/web3.js";
import type { NextPage } from "next";
import { useRouter } from "next/router";
import u from "@/styles/u.module.css";
import { WalletButton } from "@/components/WalletButton";
import { VFC } from "react";
import { MarketProvider } from "@/contexts/MarketContext";

const Home: NextPage = () => {
  const router = useRouter();
  const { base, quote } = router.query;

  let baseKey: PublicKey;
  let quoteKey: PublicKey;
  try {
    if (!(typeof base === "string") || !(typeof quote === "string")) {
      throw new Error();
    }
    baseKey = new PublicKey(base as string);
    quoteKey = new PublicKey(quote as string);
  } catch (e) {
    return (
      <div>
        Please specify a base token with and quote token with
        "?base=TOKEN-ADDRESS&#38;quote=TOKEN-ADDRESS"
      </div>
    );
  }

  return (
    <MarketProvider base={baseKey} quote={quoteKey}>
      <HomeContent />
    </MarketProvider>
  );
};

const HomeContent: VFC = () => {
  return (
    <div className={`${u["flex"]} ${u["space-between"]} ${u["flex-wrap"]}`}>
      <div className={`${u["padding-20"]} ${u["min-width-half"]}`}>
        <WalletButton
          className={`${u["full-width"]} ${u["padding-20"]} ${u["text-lg"]}`}
        />
        <div>New order</div>
        <div>Open orders</div>
      </div>
      <div className={`${u["flex-grow"]} ${u["padding-20"]}`}>Orderbook</div>
    </div>
  );
};

export default Home;
