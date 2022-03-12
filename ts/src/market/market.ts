import { getAssociatedTokenAddress, getMint, Mint } from "@solana/spl-token";
import { Connection, PublicKey, TransactionInstruction } from "@solana/web3.js";
import Decimal from "decimal.js";

import { PROGRAM_ID } from "@/consts";
import { EventFilterASTNode, SDF } from "@/eventFilter/eventFilter";
import {
  CancelOffer,
  CreateOffer,
  isCancelOffer,
  isCreateOffer,
  isMatchOffers,
  MatchOffers,
  OfferFields,
  SimpleDexEvent,
} from "@/eventFilter/eventTypes";
import { parseLog } from "@/eventFilter/parse";
import { createOfferInstruction as _createOfferInstruction } from "@/instructions";
import { AllOfferSeedsUsedError, MarketOutOfSyncError } from "@/market/err";
import { L2Entry, MarketCreateOfferOptions, Side } from "@/market/types";
import { Offer, OFFER_LAYOUT } from "@/state";

function sortHighestBidFirst(a: OfferFields, b: OfferFields): number {
  // higher: o1 / a1 > o2 / a2 => o1*a2 > o2*a1
  const aVal = a.offering * b.acceptAtLeast;
  const bVal = b.offering * a.acceptAtLeast;
  if (aVal > bVal) return -1;
  if (aVal < bVal) return 1;
  return 0;
}

function sortLowestAskFirst(a: OfferFields, b: OfferFields): number {
  // lower: a1 / o1 < a2 / o2 => a1*o2 < a2*o1
  const aVal = a.acceptAtLeast * b.offering;
  const bVal = b.acceptAtLeast * a.offering;
  if (aVal < bVal) return -1;
  if (aVal > bVal) return 1;
  return 0;
}

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

  private eventCallbacks: Map<number, (event: SimpleDexEvent) => void>;

  private eventListener: number | null;

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
    this.eventCallbacks = new Map();
    this.eventListener = null;
  }

  static async init(
    connection: Connection,
    baseTokenAddr: PublicKey,
    quoteTokenAddr: PublicKey,
    programId: PublicKey = PROGRAM_ID,
  ): Promise<Market> {
    const market = new Market(
      connection,
      baseTokenAddr,
      quoteTokenAddr,
      programId,
    );
    await market.loadBaseToken();
    await market.loadQuoteToken();
    await market.loadAllOffers();
    market.startLiveUpdates();
    return market;
  }

  public async loadBaseToken(): Promise<Mint> {
    this.baseToken = await getMint(this.connection, this.baseTokenAddr);
    return this.baseToken;
  }

  public async loadQuoteToken(): Promise<Mint> {
    this.quoteToken = await getMint(this.connection, this.quoteTokenAddr);
    return this.quoteToken;
  }

  /**
   * Load all offers for the market.
   * Note: getProgramAccounts() is very expensive for the RPC,
   * try to use this only once on initialization.
   */
  public async loadAllOffers(): Promise<Map<string, Offer>> {
    // TODO: this may be too taxing for RPC, might need to do it sequentially?
    await Promise.all([this.loadAllBids(), this.loadAllAsks()]);
    return this.offers;
  }

  public async loadAllBids(): Promise<void> {
    const allBids = await this.loadAllOfferAccountsByTokens(
      this.quoteTokenAddr,
      this.baseTokenAddr,
    );
    // sort is in-place but its fine here
    this.bidOffers = allBids
      .sort(sortHighestBidFirst)
      .map((o) => o.address.toString());
    this.upsertOffers(allBids);
  }

  public async loadAllAsks(): Promise<void> {
    const allAsks = await this.loadAllOfferAccountsByTokens(
      this.baseTokenAddr,
      this.quoteTokenAddr,
    );
    // sort is in-place but its fine here
    this.bidOffers = allAsks
      .sort(sortLowestAskFirst)
      .map((o) => o.address.toString());
    this.upsertOffers(allAsks);
  }

  private upsertOffers(offers: Offer[]): void {
    offers.forEach((offer) => {
      this.offers.set(offer.address.toString(), offer);
    });
  }

  /**
   * Loads offer from on-chain if offer missing from this.offers
   * @param offerFieldsItems
   */
  private async updateOffers(offerFieldsItems: OfferFields[]): Promise<void> {
    const promises = offerFieldsItems.map(async (offerFields) => {
      const existing = this.offers.get(offerFields.address.toString());
      if (existing) {
        const { offering, acceptAtLeast } = offerFields;
        existing.offering = offering;
        existing.acceptAtLeast = acceptAtLeast;
        return existing;
      }
      return Offer.loadByAddress(
        this.connection,
        offerFields.address,
        this.connection.commitment,
        this.programId,
      );
    });
    const updatedOffers = await Promise.all(promises);
    this.upsertOffers(updatedOffers);
  }

  private deleteOffers(offerAddrs: string[]): void {
    offerAddrs.forEach((addr) => {
      this.offers.delete(addr);
    });
  }

  /**
   * Insert an offer's pubkey into its corresponding L2 orderbook array (this.askOffers or this.bidOffers)
   * while maintaining its price sorted order.
   * Assumes pubkey is not already in the array.
   * @param offer
   * @returns
   */
  private insertSortedL2(offer: OfferFields): void {
    const isAsk = offer.acceptMint.equals(this.quoteTokenAddr);
    const [l2, sortFn] = isAsk
      ? [this.askOffers, sortLowestAskFirst]
      : [this.bidOffers, sortHighestBidFirst];
    const addrStr = offer.address.toString();
    for (let i = 0; i < l2.length; i++) {
      const existingOffer = this.offers.get(l2[i]);
      if (!existingOffer) throw new MarketOutOfSyncError();
      if (sortFn(offer, existingOffer) < 1) {
        l2.splice(i, 0, addrStr);
        return;
      }
    }
    l2.push(addrStr);
  }

  private deleteFromSortedL2(offer: OfferFields): void {
    const isAsk = offer.acceptMint.equals(this.quoteTokenAddr);
    const l2 = isAsk ? this.askOffers : this.bidOffers;
    const addrStr = offer.address.toString();
    const i = l2.indexOf(addrStr);
    if (i > -1) {
      l2.splice(i, 1);
    }
  }

  private async loadAllOfferAccountsByTokens(
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

  private isOfMarketPredicate(o: OfferFields): boolean {
    return (
      (o.acceptMint.equals(this.quoteTokenAddr) &&
        o.offerMint.equals(this.baseTokenAddr)) ||
      (o.offerMint.equals(this.quoteTokenAddr) &&
        o.acceptMint.equals(this.baseTokenAddr))
    );
  }

  private createOfferFilter(): EventFilterASTNode<SimpleDexEvent, CreateOffer> {
    return SDF.narrowType(isCreateOffer).filter(this.isOfMarketPredicate);
  }

  private registerCreateOfferCallback() {
    this.onEvent(async (event) => {
      const offerFields = this.createOfferFilter().execute(event);
      if (offerFields) {
        // load full order account from on-chain
        const offer = await Offer.loadByAddress(
          this.connection,
          offerFields.address,
          this.connection.commitment,
          this.programId,
        );
        this.insertSortedL2(offer);
        this.upsertOffers([offer]);
      }
    });
  }

  private cancelOfferFilter(): EventFilterASTNode<SimpleDexEvent, CancelOffer> {
    return SDF.narrowType(isCancelOffer).filter(this.isOfMarketPredicate);
  }

  private registerCancelOfferCallback() {
    this.onEvent((event) => {
      const offerFields = this.cancelOfferFilter().execute(event);
      if (offerFields) {
        this.deleteFromSortedL2(offerFields);
        this.deleteOffers([offerFields.address.toString()]);
      }
    });
  }

  private matchOffersFilter(): EventFilterASTNode<SimpleDexEvent, MatchOffers> {
    // if updatedOfferA is of this market, then updatedOfferB must be of this market too
    return SDF.narrowType(isMatchOffers).filter((e) =>
      this.isOfMarketPredicate(e.updatedOfferA),
    );
  }

  private registerMatchOffersCallback() {
    this.onEvent((event) => {
      const matchEvent = this.matchOffersFilter().execute(event);
      if (matchEvent) {
        const { updatedOfferA, updatedOfferB } = matchEvent;
        this.updateOffers([updatedOfferA, updatedOfferB]);
      }
    });
  }

  public onEvent(callback: (event: SimpleDexEvent) => void): number {
    let id = this.eventCallbacks.size;
    while (this.eventCallbacks.has(id)) {
      id++;
    }
    this.eventCallbacks.set(id, callback);
    return id;
  }

  public removeOnEventListener(id: number): void {
    this.eventCallbacks.delete(id);
  }

  public registerAllEventsListener(): void {
    this.eventListener = this.connection.onLogs(this.programId, (l) => {
      l.logs.forEach((log) => {
        const event = parseLog(log);
        if (event !== null) {
          Array.from(this.eventCallbacks.values()).forEach((cb) => cb(event));
        }
      });
    });
  }

  public removeAllEventsListener(): Promise<void> {
    if (this.eventListener !== null) {
      return this.connection.removeOnLogsListener(this.eventListener);
    }
    return Promise.resolve();
  }

  public startLiveUpdates(): void {
    this.registerCreateOfferCallback();
    this.registerCancelOfferCallback();
    this.registerMatchOffersCallback();
    this.registerAllEventsListener();
  }

  public getAllOffersByOwner(owner: PublicKey): Offer[] {
    return Array.from(this.offers.values()).filter((offer) =>
      offer.owner.equals(owner),
    );
  }

  public getAllBidsByOwner(owner: PublicKey): Offer[] {
    return this.getAllOffersByOwner(owner).filter((offer) =>
      offer.offerMint.equals(this.quoteTokenAddr),
    );
  }

  public getAllAsksByOwner(owner: PublicKey): Offer[] {
    return this.getAllOffersByOwner(owner).filter((offer) =>
      offer.offerMint.equals(this.baseTokenAddr),
    );
  }

  public findNextUnusedSeed(owner: PublicKey, side: Side): number {
    const allOwnerOffers =
      side === "bid"
        ? this.getAllBidsByOwner(owner)
        : this.getAllAsksByOwner(owner);
    const allSeeds = allOwnerOffers.map((offer) => offer.seed).sort();
    for (let i = 0; i < allSeeds.length - 1; i++) {
      const curr = allSeeds[i];
      const next = allSeeds[i + 1];
      if (next > curr + 1) {
        return curr + 1;
      }
    }
    const last = allSeeds[allSeeds.length - 1];
    if (last === 65535) throw new AllOfferSeedsUsedError();
    return last + 1;
  }

  public async createOfferInstruction(
    owner: PublicKey,
    side: Side,
    offering: bigint,
    acceptAtLeast: bigint,
    opts?: MarketCreateOfferOptions,
  ): Promise<TransactionInstruction> {
    const [offerMint, acceptMint] =
      side === "bid"
        ? [this.quoteTokenAddr, this.baseTokenAddr]
        : [this.baseTokenAddr, this.quoteTokenAddr];
    const acceptedOpts = opts ?? {
      payer: undefined,
      payFrom: undefined,
      refundTo: undefined,
      creditTo: undefined,
      refundRentTo: undefined,
    };
    const { payer, payFrom, refundTo, creditTo, refundRentTo } = acceptedOpts;
    const payerAddr = payer ?? owner;
    const payFromAddr =
      payFrom ?? (await getAssociatedTokenAddress(offerMint, owner, true));
    const refundToAddr = refundTo ?? payFromAddr;
    const creditToAddr =
      creditTo ?? (await getAssociatedTokenAddress(acceptMint, owner, true));
    const refundRentToAddr = refundRentTo ?? owner;
    const seed = this.findNextUnusedSeed(owner, side);
    return _createOfferInstruction(
      payerAddr,
      owner,
      payFromAddr,
      refundToAddr,
      creditToAddr,
      refundRentToAddr,
      offerMint,
      acceptMint,
      seed,
      offering,
      acceptAtLeast,
      this.programId,
    );
  }

  public getL2Bids(): Promise<L2Entry[]> {
    return this.getL2("bid");
  }

  public getL2Asks(): Promise<L2Entry[]> {
    return this.getL2("ask");
  }

  /**
   *
   * @param isBid
   * @returns
   * @throws MarketOutOfSyncError if locally cached market data is out of sync
   */
  private async getL2(side: Side): Promise<L2Entry[]> {
    const baseToken = this.baseToken ?? (await this.loadBaseToken());
    const quoteToken = this.quoteToken ?? (await this.loadQuoteToken());
    const isBid = side === "bid";
    const offerKeys = isBid ? this.bidOffers : this.askOffers;
    const baseTokenDiv = 10 ** baseToken.decimals;
    const quoteTokenDiv = 10 ** quoteToken.decimals;
    const res: L2Entry[] = [];
    offerKeys.forEach((offerKey) => {
      const offer = this.offers.get(offerKey);
      if (!offer) throw new MarketOutOfSyncError();
      const [nBaseTokens, nQuoteTokens] = isBid
        ? [offer.acceptAtLeast, offer.offering]
        : [offer.offering, offer.acceptAtLeast];
      const nQuoteDecimals = new Decimal(nQuoteTokens.toString());
      const nBaseDecimals = new Decimal(nBaseTokens.toString());
      const priceVal = nQuoteDecimals.mul(baseTokenDiv).div(nBaseDecimals);
      const priceDecimals = priceVal.div(quoteTokenDiv);
      const size = nBaseTokens;
      const sizeDecimals = nBaseDecimals.div(baseTokenDiv);
      const price = BigInt(priceVal.round().toString());
      let i = res.length - 1;
      if (res.length === 0 || res[i].price !== price) {
        res.push({
          priceDecimals,
          sizeDecimals,
          price,
          size,
        });
        i++;
      }
      res[i].size += size;
      res[i].sizeDecimals = res[i].sizeDecimals.add(sizeDecimals);
    });
    return res;
  }
}
