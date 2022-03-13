import { PublicKey } from "@solana/web3.js";

export function pubkeyAbbr(p: PublicKey): string {
  const s = p.toString();
  return `${s.substring(0, 3)}...${s.substring(s.length - 3)}`;
}

/**
 *
 * @param val
 * @param decimals
 * @returns null if val is not parseable as a token value
 */
export function parseTokenVal(val: string, decimals: number): bigint | null {
  const split = val.split(".");
  const decMultiplier = BigInt(10 ** decimals);

  try {
    switch (split.length) {
      case 1:
        return BigInt(val) * decMultiplier;
      case 2:
        return BigInt(split[0]) * decMultiplier + BigInt(split[1]);
      default:
        throw new Error("invalid token val");
    }
  } catch (e) {
    return null;
  }
}
