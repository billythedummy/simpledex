import {
  createContext,
  Dispatch,
  ReactNode,
  SetStateAction,
  useContext,
  useMemo,
  useState,
} from "react";
import { WalletAdapterNetwork } from "@solana/wallet-adapter-base";
import {
  ConnectionProvider,
  WalletProvider,
} from "@solana/wallet-adapter-react";
import { WalletModalProvider } from "@solana/wallet-adapter-react-ui";
import {
  Coin98WalletAdapter,
  PhantomWalletAdapter,
  SlopeWalletAdapter,
  SolflareWalletAdapter,
  SolletExtensionWalletAdapter,
  SolletWalletAdapter,
} from "@solana/wallet-adapter-wallets";
import { Cluster, clusterApiUrl } from "@solana/web3.js";

type SolanaContextType = {
  cluster: SolanaCluster;
  setCluster: Dispatch<SetStateAction<SolanaCluster>>;
};

type SolanaCluster = {
  label: string;
  network: Cluster;
  endpoint: string;
};

export const CLUSTERS: SolanaCluster[] = [
  {
    label: "Mainnet",
    network: "mainnet-beta",
    endpoint: clusterApiUrl("mainnet-beta"),
  },
  {
    label: "Devnet",
    network: "devnet",
    endpoint: clusterApiUrl("devnet"),
  },
];

const SolanaContext = createContext<SolanaContextType | null>(null);

export function SolanaProvider({ children }: { children: ReactNode }) {
  const [cluster, setCluster] = useState<SolanaCluster>(CLUSTERS[1]);

  const endpoint = useMemo(() => cluster.endpoint, [cluster.endpoint]);

  const wallets = useMemo(
    () => [
      new PhantomWalletAdapter(),
      new SlopeWalletAdapter(),
      new SolflareWalletAdapter({
        network: cluster.network as WalletAdapterNetwork,
      }),
      new SolletWalletAdapter({
        network: cluster.endpoint as WalletAdapterNetwork,
      }),
      new SolletExtensionWalletAdapter({
        network: cluster.network as WalletAdapterNetwork,
      }),
      new Coin98WalletAdapter(),
    ],
    [cluster.endpoint, cluster.network]
  );

  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={wallets} autoConnect>
        <WalletModalProvider>
          <SolanaContext.Provider
            value={useMemo(() => ({ cluster, setCluster }), [cluster])}
          >
            {children}
          </SolanaContext.Provider>
        </WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  );
}

export function useSolana() {
  const solanaContext = useContext(SolanaContext);

  if (!solanaContext)
    throw new Error("Make sure to wrap your app in a SolanaProvider");

  return solanaContext;
}
