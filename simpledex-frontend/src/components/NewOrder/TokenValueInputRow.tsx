import { PublicKey } from "@solana/web3.js";
import { VFC } from "react";
import u from "@/styles/u.module.css";
import { pubkeyAbbr } from "@/utils";

type TokenValueInputRowProps = {
  value: string;
  token: PublicKey;
  onChange?: (newVal: string) => void;
};

export const TokenValueInputRow: VFC<TokenValueInputRowProps> = ({
  value,
  token,
  onChange,
}) => {
  const id = `${token.toString()}-input`;

  return (
    <div
      className={`${u["flex"]} ${u["space-between"]} ${u["align-center"]} ${u["full-width"]} ${u["children-hor-margin"]}`}
    >
      <input
        className={`${u["flex-grow"]}`}
        type="text"
        id={id}
        value={value}
        onChange={(e) => {
          if (onChange) onChange(e.target.value);
        }}
      />
      <label htmlFor={id}>{pubkeyAbbr(token)} Tokens</label>
    </div>
  );
};
