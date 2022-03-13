import { useMarket } from "@/contexts/MarketContext";
import { useProvider } from "@/hooks/useProvider";
import { VFC } from "react";
import u from "@/styles/u.module.css";
import { sortBigIntAsc } from "@/utils";
import { OpenOrderRow } from "./OpenOrderRow";

export const OpenOrders: VFC = () => {
  const { wallet } = useProvider();
  const { market } = useMarket();

  const userOfferKeys = wallet.publicKey
    ? market
        .getAllOffersByOwner(wallet.publicKey)
        .sort((a, b) => sortBigIntAsc(a.slot, b.slot))
        .map((offer) => offer.address)
    : [];

  return (
    <table className={u["full-width"]}>
      <thead>
        <tr>
          <th>Side</th>
          <th>Price</th>
          <th>Size</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        {userOfferKeys.map((k) => (
          <OpenOrderRow offerKey={k} key={k.toString()} />
        ))}
      </tbody>
    </table>
  );
};
