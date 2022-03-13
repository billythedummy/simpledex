import { useMarket } from "@/contexts/MarketContext";
import { PublicKey, Transaction } from "@solana/web3.js";
import { useEffect, useMemo, useState, VFC } from "react";
import {
  CancelOffer,
  EventFilterASTNode,
  isCancelOffer,
  isMatchOffers,
  MatchOffers,
  Offer,
  SDF,
  SimpleDexEvent,
} from "simpledex";
import u from "@/styles/u.module.css";
import { sendSignConfirm, unshiftCreateATA } from "@/utils";
import { useProvider } from "@/hooks/useProvider";
import { useSolana } from "@/contexts/SolanaContext";

type OpenOrderRowProps = {
  offerKey: PublicKey;
};

function offerChangedFilter(
  offerKey: PublicKey
): EventFilterASTNode<SimpleDexEvent, CancelOffer | MatchOffers> {
  const cancelFilter = SDF.narrowType(isCancelOffer).filter((e) =>
    e.address.equals(offerKey)
  );
  const matchFilter = SDF.narrowType(isMatchOffers).filter(
    (e) =>
      e.updatedOfferA.address.equals(offerKey) ||
      e.updatedOfferB.address.equals(offerKey)
  );
  return cancelFilter.or(matchFilter);
}

export const OpenOrderRow: VFC<OpenOrderRowProps> = ({ offerKey }) => {
  const { cluster } = useSolana();
  const { wallet } = useProvider();
  const { market } = useMarket();

  const [crank, setCrank] = useState(true);
  const refetchOffer = () => setCrank(!crank);

  useEffect(() => {
    const listener = market.onEvent((e) => {
      const filter = offerChangedFilter(offerKey);
      if (filter.execute(e) !== null) refetchOffer();
    });
    return () => market.removeOnEventListener(listener);
  }, [market]);

  const offer = useMemo(
    () => market.offers.get(offerKey.toString()),
    [market, crank]
  );
  if (!offer) return null;

  const cancelOrder = async () => {
    const walletPubkey = wallet.publicKey;
    if (!walletPubkey) return;
    const ix = offer.cancelOfferInstruction();
    const tx = new Transaction();
    tx.add(ix);
    await unshiftCreateATA(
      market.connection,
      tx,
      offer.offerMint,
      walletPubkey
    );
    await sendSignConfirm(
      cluster.network,
      market.connection,
      wallet,
      tx,
      undefined,
      "Limit order cancelled"
    );
  };

  const info = market.offerMarketInfo(offer);
  const sideClass = info
    ? info.side === "bid"
      ? u["green-text"]
      : u["red-text"]
    : "";

  return (
    <tr>
      <td className={sideClass}>{info ? info.side : "-"}</td>
      <td>{info ? info.priceAndSize.priceDecimals.toFixed(4) : "-"}</td>
      <td>{info ? info.priceAndSize.sizeDecimals.toFixed(4) : "-"}</td>
      <td>
        <button onClick={cancelOrder}>Cancel</button>
      </td>
    </tr>
  );
};
