contract;

use std::{
    address::*,
    assert::assert,
    block::*,
    // chain::auth::*,
    context::{*}, 
    contract_id::ContractId,
    constants::*,
    external::bytecode_root,
    hash::*,
    result::*,
    revert::revert,
    storage::*,
    token::*,
    u128::U128,
    auth::*,
    call_frames::{contract_id, msg_asset_id},
};

use swap_abi::*;
// use token_abi::Token;
use factory_abi::Factory;
use pepaswap_helpers::*;

//TODO change api
struct SwapResult {
    amount_0_out: u64,
    amount_1_out: u64,
}

pub enum RouterError {
    NotInitialized: (), 
    MultihopPairNotFound: (),
    MaxInputExceeded: (),
    PoolFailure: (),
    AlreadyInitialized: (),
    LpTokenNotEnough: (),
    InputNotEnough: (),
  }
  

// TODO: add spec
// TODO: Add option to return token to specified address
abi Router {

    // initializer, initialize factory
    #[storage(read,write)]
    fn initialize(factory:b256);

    // add liquidity
    #[storage(read,write)]
    fn add_liquidity(swap_address:b256, amount_0:u64, amount_1:u64, amount_0_min:u64, amount_1_min:u64) -> (u64,u64);

    // remove liquidity
    // check: pair address equals lp token address
    #[storage(read,write)]
    fn remove_liquidity(swap_address:b256, amount_lp:u64, amount_0:u64, amount_1:u64, amount_0_min:u64, amount_1_min:u64) -> (u64,u64);

    // swap exact input to output
    #[storage(read,write)]
    fn swap_exact_input_for_output(swap_address:b256, asset_0:ContractId, asset_1:ContractId, asset_0_amount:u64, asset_1_amount:u64 ,amount_out_min:u64, to:Identity) -> SwapResult;


    // swap input for exact output
    // check: input amount greater than requirement, send back spare token.
    #[storage(read,write)]
    fn swap_input_for_exact_output(swap_address:b256, asset_0:ContractId, asset_1:ContractId,asset_0_amount:u64, asset_1_amount:u64, amount_in_max:u64, amount_out:u64, to:Identity) -> SwapResult;

    // swap exact input to output multihop
    #[storage(read,write)]
    fn swap_exact_input_for_output_multihop(swap_factory: b256, path:Vec<b256>, amount_in:u64 , amount_out_min:u64, to:Identity) -> SwapResult;

    // multihop swap
    #[storage(read,write)]
    fn swap_input_for_exact_output_multihop(swap_factory: b256, path:Vec<b256>, amount_out:u64 , amount_in_max:u64, to:Identity) -> SwapResult;
}


storage {
    initialized: bool = false,
    factory: b256 = ZERO_B256,
}

// swap multihop through @path with exact input
#[storage(read,write)]
fn swap_exact_input_for_output_multihop( swap_factory: b256, path:Vec<b256>, amount_in:u64 , amount_out_min:u64, to:Identity) -> SwapResult {
    require(storage.factory != ZERO_B256, RouterError::NotInitialized);
    assert(path.len() >= 2);
    let factory = abi(Factory, swap_factory);
    let mut i = 0;
    // input amount to swap contract
    let mut swap_in_amount = amount_in; 
    while i < path.len()-1 {
        // get pair address with factory
        let (address_0,address_1) = (path.get(i).unwrap(), path.get(i+1).unwrap());
        let swap_option = factory.get_swap(address_0,address_1);
        require(swap_option.is_some(), RouterError::MultihopPairNotFound);
        let swap_address = swap_option.unwrap();
        let swap = abi(Swap, swap_address);
        let (swap_token_0, swap_token_1) = swap.get_pair();
        let (amount_0, amount_1) = if address_0 == swap_token_0{
            (swap_in_amount,0) 
        } else {
            (0,swap_in_amount)
        };
        
        let result = if i+2 == path.len(){
            //Done: use u64 max until last swap
            swap_exact_input_for_output(
                swap_address,
                amount_0, 
                amount_1,
                amount_out_min,
                to
                )
        }
        else{
            // put amount out constraint on last input
            swap_exact_input_for_output(
                swap_address,
                amount_0,
                amount_1,
                u64::max(),
                to
            )
        };
        // set input amount for next swap
        swap_in_amount = if address_0 == swap_token_0  {
            result.amount_1_out 
        }else {
            result.amount_0_out
        };
        i=i+1;
    }
    let out_amount = swap_in_amount;
    SwapResult {
        amount_0_out: amount_in,
        amount_1_out: out_amount,
    }
}

// TODO: Fix error when transfer token back to "to" address
// consider removing 'to' parameter
// Follot token order in pool
#[storage(read,write)]
fn swap_exact_input_for_output(swap_address:b256,  asset_0_amount:u64, asset_1_amount:u64 ,amount_out_min:u64, to:Identity) -> SwapResult{
    require(storage.factory != ZERO_B256, RouterError::NotInitialized);
    let swap = abi(Swap,swap_address);
    let (reserve_0, reserve_1) = swap.get_reserve();
    let (token_0_address,token_1_address) = swap.get_pair();
    let token_0_amount = asset_0_amount;
    let token_1_amount = asset_1_amount;
    
    // let sender = msg_sender().unwrap();
    // // send and swap money in swap contract
    if (token_1_amount > 0) {
        force_transfer_to_contract(token_1_amount, ContractId::from(token_1_address),ContractId::from(swap_address));
        let amount_out = swap.swap();
        SwapResult {
            amount_0_out:amount_out, 
            amount_1_out:0,
        }
    } else {
        // second swap wrong
        force_transfer_to_contract(token_0_amount, ContractId::from(token_0_address),ContractId::from(swap_address));
        let amount_out = swap.swap();
        // transfer(amount_out, ContractId::from(token_1_address), to);
        SwapResult {
            amount_0_out:0, 
            amount_1_out:amount_out,
        }
    }
}

impl Router for Contract {

    // initialize router and set factory address
    #[storage(read,write)]
    fn initialize(factory:b256){
        require(storage.factory == ZERO_B256, RouterError::AlreadyInitialized);
        storage.factory = factory;
    }

    // add liquidity
    #[storage(read,write)]
    fn add_liquidity(swap_address:b256, amount_0:u64, amount_1:u64, amount_0_min:u64, amount_1_min:u64) -> (u64,u64){
        require(storage.factory != ZERO_B256, RouterError::NotInitialized);
        // TODO: get swap address from factory or 

        let swap = abi(Swap,swap_address);
        let sender = msg_sender().unwrap();
        //todo: get token amount from received liquidity
        let (reserve_0, reserve_1) = swap.get_reserve();
        let (token_0_address,token_1_address) = swap.get_pair();

        // calculate/quote
        force_transfer_to_contract(amount_0, ContractId::from(token_0_address), ContractId::from(swap_address));
        force_transfer_to_contract(amount_1, ContractId::from(token_1_address), ContractId::from(swap_address));
        let amount_1_out = swap.add_liquidity();
        transfer(amount_1_out, ContractId::from(swap_address), sender);
        (amount_0, amount_1)
    }

    // remove liquidity
    // check: pair address equals lp token address
    #[storage(read,write)]
    fn remove_liquidity(swap_address:b256,  amount_lp:u64, amount_0:u64, amount_1:u64, amount_0_min:u64, amount_1_min:u64) -> (u64,u64){
        require(storage.factory != ZERO_B256, RouterError::NotInitialized);
        // check swap_address in factory?

        let swap = abi(Swap,swap_address);
        //todo: get token amount from received liquidity
        let (reserve_0, reserve_1) = swap.get_reserve();
        let (token_0_address,token_1_address) = swap.get_pair();
        let lp_amount = this_balance(ContractId::from(swap_address));
        require(lp_amount > 0, RouterError::LpTokenNotEnough);

        // check lp token received
        let sender = msg_sender().unwrap();

        // send lp token and burn at swap and retrieve token pair
        force_transfer_to_contract(lp_amount,ContractId::from(swap_address), ContractId::from(swap_address));

        // TODO: change api
        let (token_0_amount, token_1_amount) = swap.remove_liquidity();
        require(token_0_amount > 0 && token_1_amount > 0, RouterError::PoolFailure);
        // liquidity removed, swap contract send balance to router
        // router send back to user
        let amount0 = this_balance(ContractId::from(token_0_address));
        let amount1 = this_balance(ContractId::from(token_1_address));
        transfer(amount0, ContractId::from(token_0_address), sender);
        transfer(amount1, ContractId::from(token_1_address), sender);

        // return an output structure
        (token_0_amount, token_1_amount)

    }

    // swap exact input to output
    // ignore token order
    #[storage(read,write)]
    fn swap_exact_input_for_output(swap_address:b256, asset_0:ContractId, asset_1:ContractId, asset_0_amount:u64, asset_1_amount:u64 ,amount_out_min:u64, to:Identity) -> SwapResult{
        let swap = abi(Swap,swap_address);
        //todo: get token amount from received liquidity
        let (token_0_address,token_1_address) = swap.get_pair();
        let (input_0,input_1) = if token_0_address == asset_0.into() {
            (asset_0_amount,asset_1_amount)
        }else{
            (asset_1_amount,asset_0_amount)
        };
        let swap_result = swap_exact_input_for_output(swap_address, input_0, input_1, amount_out_min, msg_sender().unwrap());
        let token_0_amount = swap_result.amount_0_out;
        let token_1_amount = swap_result.amount_1_out;
        require(token_0_amount > 0 || token_1_amount > 0, RouterError::PoolFailure);
        if token_0_amount > 0{
            transfer(token_0_amount, ContractId::from(token_0_address), msg_sender().unwrap());
        }else{
            transfer(token_1_amount, ContractId::from(token_1_address), msg_sender().unwrap());
        }
        // TODO: fix token order in result
        SwapResult {
            amount_0_out:token_0_amount, 
            amount_1_out:token_1_amount,
        }
    }
    
    // swap input for exact output
    // check: input amount greater than requirement, send back spare token.
    // No assumption for token input order
    #[storage(read,write)]
    fn swap_input_for_exact_output(swap_address:b256, asset_0:ContractId, asset_1:ContractId, asset_0_amount:u64, asset_1_amount:u64 ,  amount_in_max:u64, amount_out:u64, to:Identity) -> SwapResult{
        require(storage.factory != ZERO_B256, RouterError::NotInitialized);
        // TODO:check swap_address in factory
        let swap = abi(Swap,swap_address);
        //todo: get token amount from received liquidity
        // align token order
        let (reserve_0, reserve_1) = swap.get_reserve();
        let (token_0_address,token_1_address) = swap.get_pair();
        let (input_0,input_1, input_reserve, output_reserve) = if token_0_address == asset_0.into() {
            (asset_0_amount,asset_1_amount, reserve_0, reserve_1)
        }else{
            (asset_1_amount,asset_0_amount, reserve_1, reserve_0)
        };
        require(input_0 > 0 || input_1 > 0, RouterError::InputNotEnough);

        let sender = msg_sender().unwrap();

        //TODO: switch to use "payable" like contract input

        let mut asset_0_in = 0;
        let mut asset_1_in = 0;
        let mut output_amount = 0;
        if (input_0 > 0) {
            // input asset 0
            asset_0_in = minimum_input_given_exact_output(amount_out,input_reserve,output_reserve);
            require(asset_0_in <= amount_in_max, RouterError::MaxInputExceeded); 
            transfer(asset_0_in, ContractId::from(token_0_address), Identity::ContractId(ContractId::from(swap_address)));
            // send and swap money
            output_amount = swap.swap();
            let new_balance = input_0 - asset_0_in;
            transfer(new_balance, ContractId::from(token_0_address), sender);
            transfer(output_amount, ContractId::from(token_1_address), sender);
            SwapResult {
                amount_0_out:new_balance, 
                amount_1_out:output_amount,
            }
        } else {
            // input asset 1
            // TODO: change confusing name
            asset_1_in = minimum_input_given_exact_output(amount_out,output_reserve,input_reserve);       
            require(asset_1_in <= amount_in_max, RouterError::MaxInputExceeded); 
            transfer(asset_1_in, ContractId::from(token_1_address), Identity::ContractId(ContractId::from(swap_address)));
            // send and swap money
            output_amount = swap.swap();
            // transfer left over tokens back to user
            let new_balance = input_1 - asset_1_in;
            transfer(new_balance, ContractId::from(token_1_address), sender);
            transfer(output_amount, ContractId::from(token_0_address), sender);
            SwapResult {
                amount_0_out:output_amount, 
                amount_1_out:new_balance,
            }
        }
    }

    //TODO: add spec
    //TODO: use amount specified in parameters instead of balance
    // multihop swap
    #[storage(read,write)]
    fn swap_exact_input_for_output_multihop(swap_factory: b256, path:Vec<b256>,  amount_in:u64 , amount_out_min:u64, to:Identity) -> SwapResult {
        require(path.len() >= 2, "Path length must be greater than Two!");
        let result = swap_exact_input_for_output_multihop(swap_factory, path, amount_in , amount_out_min, msg_sender().unwrap());
        let (result_in, result_out) = (result.amount_0_out, result.amount_1_out);
        let token_out = path.get(path.len()-1).unwrap();
        require(result_out > 0, RouterError::PoolFailure);
        transfer(result_out, ContractId::from(token_out), msg_sender().unwrap());
        SwapResult {
            amount_0_out:result_in, 
            amount_1_out:result_out,
        }
    }

    //TODO: add spec
    // multihop swap
    #[storage(read,write)]
    fn swap_input_for_exact_output_multihop(swap_factory: b256, path:Vec<b256>, amount_out:u64 , amount_in_max:u64, to:Identity) -> SwapResult {
        require(storage.factory != ZERO_B256, RouterError::NotInitialized);
        require(path.len() >= 2, "Token path length must be greater than Two!");
        let factory = abi(Factory, swap_factory);
        let mut i = path.len()-2;
        // output amount to swap contract
        let mut swap_out_amount = amount_out; 
        let input_token_address = ContractId::from(path.get(0).unwrap());
        // first calculate the minimum required input then swap
        while i >0  {
            // get pair address with factory, calculating amount of token at address_0
            let (address_0,address_1) = (path.get(i).unwrap(), path.get(i+1).unwrap());
            let swap_address = factory.get_swap(address_0,address_1).unwrap();
            let swap = abi(Swap, swap_address);
            let (swap_token_0, swap_token_1) = swap.get_pair();
            let (swap_reserve_0, swap_reserve_1) = swap.get_reserve();
 
            // set reserve as well
            let (reserve_0, reserve_1) = if address_0 == swap_token_0 {
                (swap_reserve_0, swap_reserve_1)
            } else {
                (swap_reserve_1, swap_reserve_0)
            };
            
            // calculate min input
            swap_out_amount = minimum_input_given_exact_output(
                    swap_out_amount,
                    reserve_1,
                    reserve_0
                );
            i=i-1;
        }
        //TODO: add error output
        require(swap_out_amount <= amount_in_max, RouterError::MaxInputExceeded);

        // execute swap, need to change for exact output not exceeding specified amount
        let token_out = path.get(path.len()-1).unwrap();
        // TODO: Need to send token according to amount in parameter
        // TODO: might send less than swap out amount
        // TODO: use U128 during calculation
        let result = swap_exact_input_for_output_multihop(swap_factory, path, swap_out_amount, amount_out, msg_sender().unwrap());
        let (result_in, result_out) = (result.amount_0_out, result.amount_1_out);
        transfer(amount_out, ContractId::from(token_out), msg_sender().unwrap());

        SwapResult {
            amount_0_out: swap_out_amount,
            amount_1_out: amount_out,
        }
    }
}
