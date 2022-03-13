import { PublicKey } from "@solana/web3.js";
import type { NextPage } from "next";
import { useRouter } from "next/router";
import u from "@/styles/u.module.css";
import { WalletButton } from "@/components/WalletButton";
import { VFC } from "react";
import { MarketProvider, useMarket } from "@/contexts/MarketContext";
import { NewOrder } from "@/components/NewOrder";
import { useSolana } from "@/contexts/SolanaContext";
import { pubkeyAbbr } from "@/utils";

const Home: NextPage = () => {
  const router = useRouter();
  const { base, quote } = router.query;

  let baseKey: PublicKey;
  let quoteKey: PublicKey;
  try {
    if (!(typeof base === "string") || !(typeof quote === "string")) {
      throw new Error();
    }
    baseKey = new PublicKey(base);
    quoteKey = new PublicKey(quote);
  } catch (e) {
    return (
      <div>
        Please specify a base token and quote token with
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
  const { market } = useMarket();
  const { cluster } = useSolana();

  return (
    <>
      <h2 className={u["text-center"]}>
        {pubkeyAbbr(market.baseTokenAddr)} - {pubkeyAbbr(market.quoteTokenAddr)}{" "}
        Market
      </h2>
      <h3 className={u["text-center"]}>Cluster: {cluster.network}</h3>
      <div className={`${u["flex"]} ${u["space-between"]} ${u["flex-wrap"]}`}>
        <div
          className={`${u["padding-20"]} ${u["min-width-half"]} ${u["children-vert-margin"]}`}
        >
          <WalletButton
            className={`${u["full-width"]} ${u["padding-20"]} ${u["text-lg"]}`}
          />
          <NewOrder market={market} />
          <div>Open orders</div>
        </div>
        <div className={`${u["flex-grow"]} ${u["padding-20"]}`}>Orderbook</div>
      </div>
    </>
  );
};

export default Home;
