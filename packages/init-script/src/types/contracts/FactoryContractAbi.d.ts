/* Autogenerated file. Do not edit manually. */

/* tslint:disable */
/* eslint-disable */

/*
  Fuels version: 0.35.0
  Forc version: 0.35.3
  Fuel-Core version: 0.17.3
*/

import type {
  BN,
  BytesLike,
  Contract,
  DecodedValue,
  FunctionFragment,
  Interface,
  InvokeFunction,
} from 'fuels';

import type { Option, Enum } from "./common";

export type FactoryErrorInput = Enum<{ UninitailizeError: [], PoolAlreadyExist: [], TemplateMismatch: [] }>;
export type FactoryErrorOutput = FactoryErrorInput;

interface FactoryContractAbiInterface extends Interface {
  functions: {
    create_swap: FunctionFragment;
    exist_swap: FunctionFragment;
    get_swap: FunctionFragment;
    initialize: FunctionFragment;
  };

  encodeFunctionData(functionFragment: 'create_swap', values: [string]): Uint8Array;
  encodeFunctionData(functionFragment: 'exist_swap', values: [string]): Uint8Array;
  encodeFunctionData(functionFragment: 'get_swap', values: [string, string]): Uint8Array;
  encodeFunctionData(functionFragment: 'initialize', values: [string]): Uint8Array;

  decodeFunctionData(functionFragment: 'create_swap', data: BytesLike): DecodedValue;
  decodeFunctionData(functionFragment: 'exist_swap', data: BytesLike): DecodedValue;
  decodeFunctionData(functionFragment: 'get_swap', data: BytesLike): DecodedValue;
  decodeFunctionData(functionFragment: 'initialize', data: BytesLike): DecodedValue;
}

export class FactoryContractAbi extends Contract {
  interface: FactoryContractAbiInterface;
  functions: {
    create_swap: InvokeFunction<[swap_id: string], void>;
    exist_swap: InvokeFunction<[address: string], boolean>;
    get_swap: InvokeFunction<[token_0_address: string, token_1_address: string], Option<string>>;
    initialize: InvokeFunction<[template_swap_id: string], void>;
  };
}
