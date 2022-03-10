import { PublicKey } from "@solana/web3.js";

import { ParseError } from "@/eventFilter/err";
import {
  CANCEL_OFFER_EVENT_TAG,
  CancelOffer,
  CREATE_OFFER_EVENT_TAG,
  CreateOffer,
  MATCH_OFFERS_EVENT_TAG,
  MatchOffers,
  OfferFields,
  SimpleDexEvent,
} from "@/eventFilter/eventTypes";
import { isTuple, Tuple } from "@/typeUtils";

/**
 *
 * @param csv should be a csv string with
 *            [0] - offer base58
 *            [1] - offerMint base58
 *            [2] - offering
 *            [3] - acceptMint base58
 *            [4] - acceptAtLeast
 */
function parseOfferFields(fields: Tuple<string, 5>): OfferFields {
  return {
    offer: new PublicKey(fields[0]),
    offerMint: new PublicKey(fields[1]),
    offering: BigInt(fields[2]),
    acceptMint: new PublicKey(fields[3]),
    acceptAtLeast: BigInt(fields[4]),
  };
}

/**
 *
 * @param body
 * @throws if malformed event log
 */
function parseMatchOffers(body: string): MatchOffers {
  const csv = body.split(",");
  const tokenAStr = csv[0];
  const tokenBStr = csv[2];
  const updatedOfferA = parseOfferFields([
    csv[4],
    tokenAStr,
    csv[5],
    tokenBStr,
    csv[6],
  ]);
  const updatedOfferB = parseOfferFields([
    csv[7],
    tokenBStr,
    csv[8],
    tokenAStr,
    csv[9],
  ]);
  return {
    tag: MATCH_OFFERS_EVENT_TAG,
    updatedOfferA,
    updatedOfferB,
    trade: {
      tokenA: new PublicKey(tokenAStr),
      tokenB: new PublicKey(tokenBStr),
      tokenAAmount: BigInt(csv[1]),
      tokenBAmount: BigInt(csv[3]),
    },
  };
}

function parseCreateOffer(body: string): CreateOffer {
  const csv = body.split(",");
  if (!isTuple(csv, 5)) {
    throw new ParseError();
  }
  return {
    tag: CREATE_OFFER_EVENT_TAG,
    ...parseOfferFields(csv),
  };
}

function parseCancelOffer(body: string): CancelOffer {
  const csv = body.split(",");
  if (!isTuple(csv, 5)) {
    throw new ParseError();
  }
  return {
    tag: CANCEL_OFFER_EVENT_TAG,
    ...parseOfferFields(csv),
  };
}

const PROGRAM_LOG_PREFIX = "Program Log: ";

/**
 *
 * @param log the raw log string returned in @solana/web3.js:Log
 * @returns
 */
export function parseLog(log: string): SimpleDexEvent | null {
  const firstSplit = log.split(PROGRAM_LOG_PREFIX);
  if (firstSplit.length < 2) {
    return null;
  }
  const tagAndBody = firstSplit[1].split(":");
  const maybeTag = tagAndBody[0];
  // do not dereference tagAndBody[1] here, since it might throw if not event
  switch (maybeTag) {
    case CREATE_OFFER_EVENT_TAG:
      return parseCreateOffer(tagAndBody[1]);
    case MATCH_OFFERS_EVENT_TAG:
      return parseMatchOffers(tagAndBody[1]);
    case CANCEL_OFFER_EVENT_TAG:
      return parseCancelOffer(tagAndBody[1]);
    default:
      return null;
  }
}
