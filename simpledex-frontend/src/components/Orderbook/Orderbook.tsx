import { VFC } from "react";
import u from "@/styles/u.module.css";
import { OrderbookRow, OrderBookRowProps } from "./OrderbookRow";
import { useMarket } from "@/contexts/MarketContext";
import { pubkeyAbbr } from "@/utils";
import { Market } from "simpledex";
import useSWR from "swr";

const L2_DISPLAY_DEPTH = 8;

async function genOrderbook(market: Market): Promise<OrderBookRowProps[]> {
  const asks = (await market.getL2Asks()).slice(0, L2_DISPLAY_DEPTH);
  const bids = (await market.getL2Bids()).slice(0, L2_DISPLAY_DEPTH);
  const maxSizeAsk = Math.max(...asks.map((a) => a.sizeDecimals.toNumber()));
  const maxSizeBid = Math.max(...bids.map((b) => b.sizeDecimals.toNumber()));
  const maxSize = Math.max(maxSizeAsk, maxSizeBid);
  const res = asks.reverse().map((a) => ({
    price: a.priceDecimals.toNumber(),
    maxSize,
    bidSize: 0,
    askSize: a.sizeDecimals.toNumber(),
  }));
  bids.forEach((b) => {
    const price = b.priceDecimals.toNumber();
    const bidSize = b.sizeDecimals.toNumber();
    const i = res.findIndex((a) => a.price === price);
    if (i > -1) {
      res[i].bidSize = bidSize;
    } else {
      res.push({
        price,
        maxSize,
        bidSize,
        askSize: 0,
      });
    }
  });
  return res.sort((a, b) => b.price - a.price);
}

export const OrderBook: VFC = () => {
  const { market } = useMarket();

  const { data = [] } = useSWR("orderbook", () => genOrderbook(market), {
    refreshInterval: 5_000,
  });

  return (
    <div className={`${u["flex"]} ${u["flex-col"]}`}>
      <div className={`${u["flex"]} ${u["space-between"]}`}>
        <div>
          <h4>Size ({pubkeyAbbr(market.baseTokenAddr)})</h4>
        </div>
        <div>
          <h4>Price ({pubkeyAbbr(market.quoteTokenAddr)})</h4>
        </div>
      </div>
      {data.map((props) => (
        <OrderbookRow {...props} key={props.price} />
      ))}
    </div>
  );
};
