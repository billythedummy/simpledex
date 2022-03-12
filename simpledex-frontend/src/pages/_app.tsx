import { SolanaProvider } from "@/contexts/SolanaContext";
import "@/styles/globals.css";
import type { AppProps } from "next/app";

require("@solana/wallet-adapter-react-ui/styles.css");

function MyApp({ Component, pageProps }: AppProps) {
  return (
    <SolanaProvider>
      <Component {...pageProps} />
    </SolanaProvider>
  );
}

export default MyApp;
