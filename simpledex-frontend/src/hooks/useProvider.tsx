import { WalletName } from "@solana/wallet-adapter-base";
import { useConnection, useWallet } from "@solana/wallet-adapter-react";

export const useProvider = () => {
  const { connection } = useConnection();
  const wallet = useWallet();

  const selectWallet = (walletName: WalletName) => {
    try {
      wallet.select(walletName);
    } catch (e) {
      // TODO: handle error
      console.log(e);
      alert(`Failed to select wallet ${walletName}`);
    }
  };

  return {
    connection,
    wallet,
    selectWallet,
  };
};
