import { bn, WalletUnlocked } from "fuels";

import "../../load.envs";
// import './loadDockerEnv';
import {
  SwapContractAbi__factory,
  TokenContractAbi__factory,
  RouterContractAbi__factory,
  FactoryContractAbi__factory,
} from "../../src/types/contracts";

import { initializeFactory } from "./initializeFactory";
import { initializeRouter } from "./initializeRouter";
import { initializePool } from "./initializePool";
import { initializeTokenContract } from "./initializeTokenContract";
import { getWalletInstance, getAccountInstance } from "./getWalletInstance";

const {
  WALLET_SECRET,
  PROVIDER_URL,
  GAS_PRICE,
  VITE_CONTRACT_ID,
  VITE_TOKEN_ID,
  VITE_ROUTER_ID,
  VITE_FACTORY_ID,
} = process.env;

if (!WALLET_SECRET) {
  process.stdout.write("WALLET_SECRET is not detected!\n");
  process.exit(1);
}

if (!VITE_CONTRACT_ID || !VITE_TOKEN_ID) {
  process.stdout.write("CONTRACT_ID or TOKEN_ID is not detected!\n");
  process.exit(1);
}
console.log(WALLET_SECRET);

async function main() {
  const wallet = await getWalletInstance();
  const account = await getAccountInstance();
  const tokenContract = TokenContractAbi__factory.connect(
    VITE_TOKEN_ID!,
    wallet
  );
  const exchangeContract = SwapContractAbi__factory.connect(
    VITE_CONTRACT_ID!,
    wallet
  );
  const factoryContract = FactoryContractAbi__factory.connect(
    VITE_FACTORY_ID!,
    wallet
  );
  const routerContract = RouterContractAbi__factory.connect(
    VITE_ROUTER_ID!,
    wallet
  );
  // exchangeContract.account = account;
  const overrides = {
    gasPrice: bn(GAS_PRICE || 0),
  };

  console.log("Initialization start");

  await initializeTokenContract(tokenContract, overrides);
  await initializePool(tokenContract, exchangeContract, overrides);
  await initializeFactory(
    tokenContract,
    exchangeContract,
    factoryContract,
    overrides
  );
  await initializeRouter(
    tokenContract,
    exchangeContract,
    factoryContract,
    routerContract,
    overrides
  );

  console.log("Initialization complete");
}

main();
