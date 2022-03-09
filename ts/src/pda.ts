import { u16 } from "@solana/buffer-layout";
import { PublicKey } from "@solana/web3.js";

import { PROGRAM_ID } from "@/consts";

function u16ToBuffer(n: number): Uint8Array {
  const res = Buffer.alloc(2);
  u16().encode(n, res);
  return res;
}

export function findOfferPda(
  owner: PublicKey,
  offerMint: PublicKey,
  acceptMint: PublicKey,
  seed: number,
  programId: PublicKey = PROGRAM_ID,
): Promise<[PublicKey, number]> {
  return PublicKey.findProgramAddress(
    [
      owner.toBuffer(),
      offerMint.toBuffer(),
      acceptMint.toBuffer(),
      u16ToBuffer(seed),
    ],
    programId,
  );
}

export function createOfferPda(
  owner: PublicKey,
  offerMint: PublicKey,
  acceptMint: PublicKey,
  seed: number,
  bump: number,
  programId: PublicKey = PROGRAM_ID,
): Promise<PublicKey> {
  return PublicKey.createProgramAddress(
    [
      owner.toBuffer(),
      offerMint.toBuffer(),
      acceptMint.toBuffer(),
      u16ToBuffer(seed),
      Buffer.from([bump]),
    ],
    programId,
  );
}
