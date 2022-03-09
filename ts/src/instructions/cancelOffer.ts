import { u8 } from "@solana/buffer-layout";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { PublicKey, TransactionInstruction } from "@solana/web3.js";

import { PROGRAM_ID } from "@/consts";
import { SimpleDexInstruction } from "@/instructions/types";
import { Offer } from "@/state/offer";

export function cancelOfferInstruction(
  offer: Offer,
  programId: PublicKey = PROGRAM_ID,
): TransactionInstruction {
  const keys = [
    { pubkey: offer.owner, isSigner: true, isWritable: false },
    { pubkey: offer.address, isSigner: false, isWritable: true },
    { pubkey: offer.holdingAddress, isSigner: false, isWritable: true },
    { pubkey: offer.refundTo, isSigner: false, isWritable: true },
    { pubkey: offer.refundRentTo, isSigner: false, isWritable: true },
    { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
  ];

  const data = Buffer.alloc(1);
  u8().encode(SimpleDexInstruction.CancelOffer, data);

  return new TransactionInstruction({ keys, programId, data });
}
