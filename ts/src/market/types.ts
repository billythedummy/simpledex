import { PublicKey } from "@solana/web3.js";
import Decimal from "decimal.js";

/**
 * price is in quote token
 * size is in base token
 */
export type L2Entry = {
  /**
   * price in quote token decimals.
   */
  priceDecimals: Decimal;
  /**
   * size in base token decimals.
   */
  sizeDecimals: Decimal;
  /**
   * price in quote token atomics
   * per 1.0 base token
   */
  price: bigint;
  /**
   * size in base token atomics
   */
  size: bigint;
};

export type Side = "bid" | "ask";

export type MarketCreateOfferOptions = {
  payer?: PublicKey;
  payFrom?: PublicKey;
  refundTo?: PublicKey;
  creditTo?: PublicKey;
  refundRentTo?: PublicKey;
};

export type MarketOfferInfo = {
  side: Side;
  priceAndSize: L2Entry;
};
