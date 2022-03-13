import { struct, u8, u16 } from "@solana/buffer-layout";
import { u64 } from "@solana/buffer-layout-utils";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionInstruction,
} from "@solana/web3.js";

import { PROGRAM_ID } from "@/consts";
import { SimpleDexInstruction } from "@/instructions/types";
import { findOfferPda } from "@/pda";
import { Offer } from "@/state";

export type CreateOfferArgs = {
  bump: number;
  seed: number;
  offering: bigint;
  acceptAtLeast: bigint;
};

export type CreateOfferInstructionData = {
  instruction: SimpleDexInstruction.CreateOffer;
  args: CreateOfferArgs;
};

export const CREATE_OFFER_INSTRUCTION_DATA = struct<CreateOfferInstructionData>(
  [
    u8("instruction"),
    struct<CreateOfferArgs>(
      [u8("bump"), u16("seed"), u64("offering"), u64("acceptAtLeast")],
      "args",
    ),
  ],
);

export async function createOfferInstruction(
  payer: PublicKey,
  owner: PublicKey,
  payFrom: PublicKey,
  refundTo: PublicKey,
  creditTo: PublicKey,
  refundRentTo: PublicKey,
  offerMint: PublicKey,
  acceptMint: PublicKey,
  seed: number,
  offering: bigint,
  acceptAtLeast: bigint,
  programId: PublicKey = PROGRAM_ID,
): Promise<TransactionInstruction> {
  const [offer, bump] = await findOfferPda(
    owner,
    offerMint,
    acceptMint,
    seed,
    programId,
  );
  const holding = await Offer.holdingAddress(offerMint, offer);

  const keys = [
    { pubkey: payer, isSigner: true, isWritable: true },
    { pubkey: owner, isSigner: true, isWritable: false },
    { pubkey: payFrom, isSigner: false, isWritable: true },
    { pubkey: offer, isSigner: false, isWritable: true },
    { pubkey: holding, isSigner: false, isWritable: true },
    { pubkey: refundTo, isSigner: false, isWritable: false },
    { pubkey: creditTo, isSigner: false, isWritable: false },
    { pubkey: refundRentTo, isSigner: false, isWritable: false },
    { pubkey: offerMint, isSigner: false, isWritable: false },
    { pubkey: acceptMint, isSigner: false, isWritable: false },
    { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    { pubkey: ASSOCIATED_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    // TODO: remove once ata 1.0.5 drops
    { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
  ];

  const data = Buffer.alloc(CREATE_OFFER_INSTRUCTION_DATA.span);
  CREATE_OFFER_INSTRUCTION_DATA.encode(
    {
      instruction: SimpleDexInstruction.CreateOffer,
      args: {
        bump,
        seed,
        offering,
        acceptAtLeast,
      },
    },
    data,
  );

  return new TransactionInstruction({ keys, programId, data });
}
