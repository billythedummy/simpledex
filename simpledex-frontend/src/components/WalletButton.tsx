import { useWalletModal } from "@solana/wallet-adapter-react-ui";
import { useProvider } from "@/hooks/useProvider";
import { ButtonHTMLAttributes } from "react";

type WalletButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  /**
   * Text to be displayed on the button.
   * @default "Connect"
   */
  text?: string;
};

export function WalletButton({
  text = "Connect Wallet",
  ...buttonStyles
}: WalletButtonProps) {
  const { wallet } = useProvider();
  const { setVisible, visible } = useWalletModal();

  const openConnectWalletModal = () => setVisible(!visible);

  const disconnectWallet = async () => {
    try {
      await wallet.disconnect();
    } catch (e) {
      // TODO: handle error with toast
      console.log(e);
    }
  };

  return (
    <button
      {...buttonStyles}
      onClick={wallet.connected ? disconnectWallet : openConnectWalletModal}
    >
      {wallet.connected ? `${wallet.publicKey?.toString()}` : text}
    </button>
  );
}
