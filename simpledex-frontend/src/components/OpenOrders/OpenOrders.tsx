import { useMarket } from "@/contexts/MarketContext";
import { useProvider } from "@/hooks/useProvider";
import { useEffect, VFC } from "react";
import u from "@/styles/u.module.css";
import { sortBigIntAsc } from "@/utils";
import { OpenOrderRow } from "./OpenOrderRow";
import { isCreateOffer, SDF } from "simpledex";
import useSWR, { useSWRConfig } from "swr";

const USER_OFFER_KEYS_KEY = "userOfferKeys";

export const OpenOrders: VFC = () => {
  const { wallet } = useProvider();
  const { market } = useMarket();
  const { mutate } = useSWRConfig();

  const { data: userOfferKeys = [] } = useSWR(
    [USER_OFFER_KEYS_KEY, market],
    (_key, marketArg) =>
      wallet.publicKey
        ? marketArg
            .getAllOffersByOwner(wallet.publicKey)
            .sort((a, b) => sortBigIntAsc(a.slot, b.slot))
            .map((offer) => offer.address)
        : []
  );

  useEffect(() => {
    const walletPubkey = wallet.publicKey;
    if (!walletPubkey) return;
    const listener = market.onEvent((e) => {
      // cant filter by owner here since thats not part of the logs
      const filter = SDF.narrowType(isCreateOffer).filter(
        market.getIsOfMarketPredicate()
      );
      const offerFields = filter.execute(e);
      if (offerFields !== null) {
        // this is very hacky: give time for market to fetch data from on-chain
        // i could get the data over from NewOrder to here instead...
        setInterval(() => {
          const offer = market.offers.get(offerFields.address.toString());
          if (offer && offer.owner.equals(walletPubkey)) {
            mutate([USER_OFFER_KEYS_KEY, market]);
          }
        }, 15_000);
      }
    });
    return () => market.removeOnEventListener(listener);
  }, [wallet, market]);

  return (
    <div
      className={`${u["full-width"]} ${u["max-height-400"]} ${u["overflow-y-scroll"]}`}
    >
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
    </div>
  );
};
