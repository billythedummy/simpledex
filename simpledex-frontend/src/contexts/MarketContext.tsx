import { useConnection } from "@solana/wallet-adapter-react";
import { PublicKey } from "@solana/web3.js";
import { createContext, FC, useContext, useMemo } from "react";
import { Market } from "simpledex";
import useSWR from "swr";

type MarketContextType = {
  market: Market;
};

type MarketProviderArgs = {
  base: PublicKey;
  quote: PublicKey;
};

const MarketContext = createContext<MarketContextType | null>(null);

async function initMarket(market: Market): Promise<void> {
  await market.loadAll();
}

export const MarketProvider: FC<MarketProviderArgs> = ({
  base,
  quote,
  children,
}) => {
  const { connection } = useConnection();
  const market = useMemo(
    () => new Market(connection, base, quote),
    [connection, base, quote]
  );

  useSWR(market, initMarket, {
    revalidateIfStale: false,
    revalidateOnFocus: false,
    refreshInterval: 0,
  });

  return (
    <MarketContext.Provider value={{ market }}>
      {children}
    </MarketContext.Provider>
  );
};

export function useMarket() {
  const marketContext = useContext(MarketContext);

  if (!marketContext)
    throw new Error("Make sure to wrap your page in a MarketProvider");

  return marketContext;
}
