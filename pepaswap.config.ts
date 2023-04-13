import dotenv from "dotenv";
import { createConfig, replaceEventOnEnv } from "swayswap-scripts";

const { NODE_ENV, OUTPUT_ENV } = process.env;

function getEnvName() {
  return NODE_ENV === "test" ? ".env.test" : ".env";
}

dotenv.config({
  path: `./${getEnvName()}`,
});

const getDeployOptions = () => ({
  gasPrice: Number(process.env.GAS_PRICE || 0),
});

// building types and contracts
export default createConfig({
  types: {
    artifacts: "./contracts/**/out/debug/**-abi.json",
    output: "./packages/init-script/src/types/contracts",
  },
  contracts: [
    {
      name: "VITE_TOKEN_ID",
      path: "./contracts/token_contract",
      options: getDeployOptions(),
    },
    {
      name: "VITE_TOKEN_ID2",
      path: "./contracts/token_contract",
      options: getDeployOptions(),
    },
    {
      name: "VITE_CONTRACT_ID",
      path: "./contracts/swap_contract",
      options: (contracts) => {
        const contractDeployed = contracts.find(
          (c) => c.name === "VITE_TOKEN_ID"
        )!;
        return {
          ...getDeployOptions(),
          storageSlots: [
            {
              key: "0x0000000000000000000000000000000000000000000000000000000000000001",
              value: contractDeployed.contractId,
            },
          ],
        };
      },
    },
    {
      name: "VITE_FACTORY_ID",
      path: "./contracts/factory_contract",
      options: getDeployOptions(),
    },
    {
      name: "VITE_ROUTER_ID",
      path: "./contracts/router_contract",
      options: getDeployOptions(),
    },
  ],
  onSuccess: (event) => {
    replaceEventOnEnv(
      `./packages/init-script/${OUTPUT_ENV || getEnvName()}`,
      event
    );
  },
});
