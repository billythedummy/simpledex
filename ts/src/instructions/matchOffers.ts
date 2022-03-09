import { u8 } from "@solana/buffer-layout";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { PublicKey, TransactionInstruction } from "@solana/web3.js";

import { PROGRAM_ID } from "@/consts";
import { SimpleDexInstruction } from "@/instructions/types";
import { Offer } from "@/state";

export function matchOffersInstruction(
  offerA: Offer,
  offerB: Offer,
  matcherATokenAccount: PublicKey,
  matcherBTokenAccount: PublicKey,
  programId: PublicKey = PROGRAM_ID,
): TransactionInstruction {
  const keys = [
    { pubkey: offerA.address, isSigner: false, isWritable: true },
    { pubkey: offerA.holdingAddress, isSigner: false, isWritable: true },
    { pubkey: offerB.address, isSigner: false, isWritable: true },
    { pubkey: offerB.holdingAddress, isSigner: false, isWritable: true },
    { pubkey: offerA.creditTo, isSigner: false, isWritable: true },
    { pubkey: offerA.refundTo, isSigner: false, isWritable: true },
    { pubkey: offerA.refundRentTo, isSigner: false, isWritable: true },
    { pubkey: offerB.creditTo, isSigner: false, isWritable: true },
    { pubkey: offerB.refundTo, isSigner: false, isWritable: true },
    { pubkey: offerB.refundRentTo, isSigner: false, isWritable: true },
    { pubkey: matcherATokenAccount, isSigner: false, isWritable: true },
    { pubkey: matcherBTokenAccount, isSigner: false, isWritable: true },
    { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
  ];

  const data = Buffer.alloc(1);
  u8().encode(SimpleDexInstruction.CancelOffer, data);

  return new TransactionInstruction({ keys, programId, data });
}
