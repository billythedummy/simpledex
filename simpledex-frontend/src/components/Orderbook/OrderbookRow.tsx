import { VFC } from "react";
import u from "@/styles/u.module.css";

export type OrderBookRowProps = {
  price: number;
  maxSize: number;
  bidSize: number;
  askSize: number;
};

export const OrderbookRow: VFC<OrderBookRowProps> = ({
  price,
  maxSize,
  bidSize,
  askSize,
}) => {
  const bidSizeStr = bidSize.toFixed(4);
  const askSizeStr = askSize.toFixed(4);
  let sizeDisplay;
  if (bidSize && askSize) {
    sizeDisplay = `${bidSizeStr} / ${askSizeStr}`;
  } else if (bidSize) {
    sizeDisplay = bidSizeStr;
  } else {
    sizeDisplay = askSizeStr;
  }

  return (
    <div className={`${u["flex"]} ${u["space-between"]}`}>
      <div>
        <strong>{sizeDisplay}</strong>
      </div>
      <div className={`${u["width-80-pct"]} ${u["pos-rel"]}`}>
        <div className={`${u["text-right"]} ${u["z-3"]}`}>
          <strong>{price.toFixed(4)}</strong>
        </div>
        <div
          className={`${u["pos-abs"]} ${u["translucent"]} ${u["height-100-pct"]} ${u["top-zero"]} ${u["right-zero"]} ${u["green-bg"]} ${u["z-2"]}`}
          style={{ width: `${(bidSize / maxSize) * 100}%` }}
        />
        <div
          className={`${u["pos-abs"]} ${u["translucent"]} ${u["height-100-pct"]} ${u["top-zero"]} ${u["right-zero"]} ${u["red-bg"]} ${u["z-1"]}`}
          style={{ width: `${(askSize / maxSize) * 100}%` }}
        />
      </div>
    </div>
  );
};
