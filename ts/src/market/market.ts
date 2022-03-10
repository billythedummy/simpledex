import { getMint, Mint } from "@solana/spl-token";
import { Connection, PublicKey } from "@solana/web3.js";

import { PROGRAM_ID } from "@/consts";
import { Offer, OFFER_LAYOUT } from "@/state";

/**
 *
 */
export class Market {
  /**
   * Map base-58 of the offer address to the Offer struct
   */
  public offers: Map<string, Offer>;

  /**
   * base-58 of the bid offers, in descending order of price
   * (highest bidder first)
   */
  public bidOffers: string[];

  /**
   * base-58 of the ask offers, in ascending order of price
   * (lowest asker first)
   */
  public askOffers: string[];

  public baseToken: Mint | null;

  public quoteToken: Mint | null;

  /**
   *
   * @param connection
   * @param baseToken size is in base tokens. Asks are offers offering baseTokens for quoteTokens
   * @param quoteToken price is in quote tokens. Bids are offers offering quoteTokens for baseTokens
   * @param programId
   */
  constructor(
    public readonly connection: Connection,
    public readonly baseTokenAddr: PublicKey,
    public readonly quoteTokenAddr: PublicKey,
    public readonly programId: PublicKey = PROGRAM_ID,
  ) {
    this.offers = new Map();
    this.bidOffers = [];
    this.askOffers = [];
    this.baseToken = null;
    this.quoteToken = null;
  }

  async loadBaseToken(): Promise<Mint> {
    this.baseToken = await getMint(this.connection, this.baseTokenAddr);
    return this.baseToken;
  }

  async loadQuoteToken(): Promise<Mint> {
    this.quoteToken = await getMint(this.connection, this.quoteTokenAddr);
    return this.quoteToken;
  }

  /**
   * Load all offers for the market.
   * Note: getProgramAccounts() is very expensive for the RPC,
   * try to use this only once on initialization.
   */
  async loadAllOffers(): Promise<Map<string, Offer>> {
    // TODO: this may be too taxing for RPC, might need to do it sequentially?
    await Promise.all([this.loadAllBids(), this.loadAllAsks()]);
    return this.offers;
  }

  async loadAllBids(): Promise<void> {
    const allBids = await this.loadAllOfferAccountsByTokens(
      this.quoteTokenAddr,
      this.baseTokenAddr,
    );
    const prices = Market.offersToPriceInfo(allBids);
    // higher: o1 / a1 > o2 / a2 => o1*a2 > o2*a1
    this.bidOffers = prices
      .sort((a, b) => {
        const aVal = a.offering * b.acceptAtLeast;
        const bVal = b.offering * a.acceptAtLeast;
        if (aVal > bVal) return -1;
        if (aVal < bVal) return 1;
        return 0;
      })
      .map((o) => o.address.toString());
    this.upsertOffers(allBids);
  }

  async loadAllAsks(): Promise<void> {
    const allAsks = await this.loadAllOfferAccountsByTokens(
      this.baseTokenAddr,
      this.quoteTokenAddr,
    );
    const prices = Market.offersToPriceInfo(allAsks);
    // lower: a1 / o1 < a2 / o2 => a1*o2 < a2*o1
    this.bidOffers = prices
      .sort((a, b) => {
        const aVal = a.acceptAtLeast * b.offering;
        const bVal = b.acceptAtLeast * a.offering;
        if (aVal < bVal) return -1;
        if (aVal > bVal) return 1;
        return 0;
      })
      .map((o) => o.address.toString());
    this.upsertOffers(allAsks);
  }

  private upsertOffers(offers: Offer[]): void {
    offers.forEach((offer) => {
      this.offers.set(offer.address.toString(), offer);
    });
  }

  private static offersToPriceInfo(
    offers: Offer[],
  ): { address: PublicKey; offering: bigint; acceptAtLeast: bigint }[] {
    return offers.map((offer) => ({
      address: offer.address,
      offering: offer.offering,
      acceptAtLeast: offer.acceptAtLeast,
    }));
  }

  async loadAllOfferAccountsByTokens(
    offerMint: PublicKey,
    acceptMint: PublicKey,
  ): Promise<Offer[]> {
    const accs = await this.connection.getProgramAccounts(this.programId, {
      filters: [
        {
          memcmp: {
            bytes: offerMint.toBase58(),
            offset: OFFER_LAYOUT.offsetOf("offerMint")!,
          },
        },
        {
          memcmp: {
            bytes: acceptMint.toBase58(),
            offset: OFFER_LAYOUT.offsetOf("acceptMint")!,
          },
        },
      ],
    });
    const allOffersPromises = accs.map(
      async ({ pubkey, account: { data } }) => {
        const rawOffer = OFFER_LAYOUT.decode(data);
        const holdingAddress = await Offer.holdingAddress(offerMint, pubkey);
        return new Offer(rawOffer, pubkey, holdingAddress);
      },
    );
    return Promise.all(allOffersPromises);
  }

  public getOffers(): Map<string, Offer> {
    return this.offers;
  }
}
