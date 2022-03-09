import { PublicKey } from "@solana/web3.js";

export const CREATE_OFFER_EVENT_TAG = "CREATE";
export const CANCEL_OFFER_EVENT_TAG = "CANCEL";
export const MATCH_OFFERS_EVENT_TAG = "MATCH";

export type CreateOfferEventTag = typeof CREATE_OFFER_EVENT_TAG;
export type CancelOfferEventTag = typeof CANCEL_OFFER_EVENT_TAG;
export type MatchOffersEventTag = typeof MATCH_OFFERS_EVENT_TAG;

export type EventTypeTag =
  | CreateOfferEventTag
  | CancelOfferEventTag
  | MatchOffersEventTag;

export interface OfferFields {
  offer: PublicKey;
  offerMint: PublicKey;
  offering: bigint;
  acceptMint: PublicKey;
  acceptAtLeast: bigint;
}

export type CreateOffer = OfferFields & { tag: CreateOfferEventTag };

export type CancelOffer = OfferFields & { tag: CancelOfferEventTag };

export type MatchOffers = {
  tag: MatchOffersEventTag;
  updatedOfferA: OfferFields;
  updatedOfferB: OfferFields;
  trade: {
    tokenA: PublicKey;
    tokenB: PublicKey;
    tokenAAmount: bigint;
    tokenBAmount: bigint;
  };
};

export type SimpleDexEvent = CreateOffer | CancelOffer | MatchOffers;
