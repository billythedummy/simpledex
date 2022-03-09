import { struct, u8, u16 } from "@solana/buffer-layout";
import { publicKey, u64 } from "@solana/buffer-layout-utils";
import { getAssociatedTokenAddress } from "@solana/spl-token";
import {
  Commitment,
  Connection,
  PublicKey,
  TransactionInstruction,
} from "@solana/web3.js";

import { PROGRAM_ID } from "@/consts";
import { cancelOfferInstruction as _cancelOfferInstruction } from "@/instructions/cancelOffer";
import { matchOffersInstruction as _matchOffersInstruction } from "@/instructions/matchOffers";
import { createOfferPda, findOfferPda } from "@/pda";
import {
  OfferAccountInvalidOwnerError,
  OfferAccountInvalidSizeError,
  OfferNotFoundError,
} from "@/state/err";

export interface RawOffer {
  slot: bigint;
  offering: bigint;
  acceptAtLeast: bigint;
  seed: number;
  bump: number;
  owner: PublicKey;
  offerMint: PublicKey;
  acceptMint: PublicKey;
  refundTo: PublicKey;
  creditTo: PublicKey;
  refundRentTo: PublicKey;
}

export const OFFER_LAYOUT = struct<RawOffer>([
  u64("slot"),
  u64("offering"),
  u64("acceptAtLeast"),
  u16("seed"),
  u8("bump"),
  publicKey("owner"),
  publicKey("offerMint"),
  publicKey("acceptMint"),
  publicKey("refundTo"),
  publicKey("creditTo"),
  publicKey("refundRentTo"),
]);

export const OFFER_SIZE = OFFER_LAYOUT.span;

export class Offer implements RawOffer {
  public readonly slot: bigint;

  public readonly offering: bigint;

  public readonly acceptAtLeast: bigint;

  public readonly seed: number;

  public readonly bump: number;

  public readonly owner: PublicKey;

  public readonly offerMint: PublicKey;

  public readonly acceptMint: PublicKey;

  public readonly refundTo: PublicKey;

  public readonly creditTo: PublicKey;

  public readonly refundRentTo: PublicKey;

  // cache PDA and ATA
  public readonly address: PublicKey;

  public readonly holdingAddress: PublicKey;

  private constructor(
    rawOffer: RawOffer,
    address: PublicKey,
    holdingAddress: PublicKey,
  ) {
    Object.assign(this, rawOffer);
    this.address = address;
    this.holdingAddress = holdingAddress;
  }

  static async loadByAddress(
    connection: Connection,
    addr: PublicKey,
    commitment?: Commitment,
    programId: PublicKey = PROGRAM_ID,
  ): Promise<Offer> {
    const info = await connection.getAccountInfo(addr, commitment);
    if (!info) throw new OfferNotFoundError();
    if (!info.owner.equals(programId))
      throw new OfferAccountInvalidOwnerError();
    if (info.data.length < OFFER_SIZE) throw new OfferAccountInvalidSizeError();

    const rawOffer = OFFER_LAYOUT.decode(info.data.slice(0, OFFER_SIZE));

    const address = await createOfferPda(
      rawOffer.owner,
      rawOffer.offerMint,
      rawOffer.acceptMint,
      rawOffer.seed,
      rawOffer.bump,
    );

    const holdingAddress = await Offer.holdingAddress(
      rawOffer.offerMint,
      address,
    );

    return new Offer(rawOffer, address, holdingAddress);
  }

  static async load(
    connection: Connection,
    owner: PublicKey,
    offerMint: PublicKey,
    acceptMint: PublicKey,
    seed: number,
    commitment?: Commitment,
    programId: PublicKey = PROGRAM_ID,
  ): Promise<Offer> {
    const [pda] = await findOfferPda(
      owner,
      offerMint,
      acceptMint,
      seed,
      programId,
    );
    return Offer.loadByAddress(connection, pda, commitment, programId);
  }

  static holdingAddress(
    offerMint: PublicKey,
    offerAddress: PublicKey,
  ): Promise<PublicKey> {
    return getAssociatedTokenAddress(offerMint, offerAddress, true);
  }

  cancelOfferInstruction(
    programId: PublicKey = PROGRAM_ID,
  ): TransactionInstruction {
    return _cancelOfferInstruction(this, programId);
  }

  matchOffersInstruction(
    other: Offer,
    matcherOfferTokenAccount: PublicKey,
    matcherAcceptTokenAccount: PublicKey,
    programId: PublicKey = PROGRAM_ID,
  ): TransactionInstruction {
    return _matchOffersInstruction(
      this,
      other,
      matcherOfferTokenAccount,
      matcherAcceptTokenAccount,
      programId,
    );
  }
}
