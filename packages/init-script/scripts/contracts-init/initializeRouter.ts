import type { BigNumberish } from "fuels";
import { bn, NativeAssetId } from "fuels";
import { Contract } from "fuels";

import {
  RouterContractAbi,
  SwapContractAbi,
  FactoryContractAbi,
} from "../../src/types/contracts";
import type { TokenContractAbi } from "../../src/types/contracts";

const { TOKEN_AMOUNT, ETH_AMOUNT } = process.env;
export async function initializeRouter(
  tokenContract: TokenContractAbi,
  exchangeContract: SwapContractAbi,
  factoryContract: FactoryContractAbi,
  routerContract: RouterContractAbi,
  overrides: { gasPrice: BigNumberish }
) {
  const account = exchangeContract.account!;
  const tokenAmountMint = bn(TOKEN_AMOUNT || "0x44360000");
  const tokenAmount = bn(TOKEN_AMOUNT || "0x40000");
  const ethAmount = bn(ETH_AMOUNT || "0xAAAA00");
  const address = {
    value: account.address.toB256(),
  };
  // clean up after initialization to avoid unassigned id
  const tokenId = {
    value: tokenContract.id.toB256(),
  };
  const NativeAsset = {
    value: NativeAssetId,
  };

  process.stdout.write("Initialize router\n");
  const deadline = await account.provider.getBlockNumber();
  await routerContract
    .multiCall([
      // use custom struct support wrapping parameters
      routerContract.functions.initialize(factoryContract.id.toB256()),
      //   exchangeContract.functions.deposit().callParams({
      //     forward: [ethAmount, NativeAssetId],
      //   }),
      //   exchangeContract.functions.deposit().callParams({
      //     forward: [tokenAmount, tokenContract.id.toB256()],
      //   }),
      //   exchangeContract.functions.add_liquidity()
      //   .callParams({
      //     forward: [tokenAmount, tokenContract.id.toB256()],
      //   }),
    ])
    .txParams({
      ...overrides,
      variableOutputs: 2,
      gasLimit: 100_000_000,
    })
    .addContracts([exchangeContract as Contract, factoryContract as Contract])
    .call();
}
