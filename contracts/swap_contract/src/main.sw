contract;

use std::{
    address::Address,
    assert::assert,
    block::*,
    context::{*},
    contract_id::ContractId,
    constants::ZERO_B256,
    external::bytecode_root,
    hash::*,
    result::*,
    revert::revert,
    storage::*,
    token::*,
    u128::U128,
    constants::*,
    auth::*,
    call_frames::{contract_id, msg_asset_id},
};

use pepaswap_helpers::*;
use swap_abi::Swap;
use token_abi::Token;
use factory_abi::Factory;


const MINIMUM_LIQUIDITY:U128 = U128::from((0,1000));
const DECIMALS:u64 = 9 ;
const SCALE:u64 = 1_000_000_000;
const FIVE_U128:U128 = U128::from((0,5));

fn initial_liquidity(a:u64,b:u64)->u64 {
    let prod:U128  = U128::from((0, a))*U128::from((0, b));
    require( prod  > MINIMUM_LIQUIDITY , SwapError::MinimumLiquidityNotMet);
    (prod-MINIMUM_LIQUIDITY).sqrt().as_u64().unwrap()
}

pub enum SwapError {
    MinimumLiquidityNotMet: (),
    NotInitialized: (),
    ReinitializeError: (),
    InputNotEnough: (),
    OutputNotEnough: (),
    InvalidAsset: (),
    InvalidAmount: (),
    InvalidDeadline: (),
    LpTokenNotEnough: (),
    PoolLiquidityNotEnough: (),
    LiquidityRemovedNotEnough: (),
}

storage {
    //reserves
    token_0_address: Option<ContractId> = Option::None,
    token_1_address:Option<ContractId> = Option::None,
    factory_address:b256 = ZERO_B256,
    reserve_0:u64 = 0,
    reserve_1:u64 = 0,
    lp_supply : u64 = 0, // total number of LP available
    token_supply: u64 = 0, // Total supply of LP Tokens

    //limitation
    min_asset_0:u64 = 0,
    min_asset_1:u64 = 0,

}


// mint liquidity equivalent to 1/6th of the growth in sqrt(k)
#[storage(read, write)]
fn _mintFee(reserve_0: u64, reserve_1:u64) {
    let factory = abi(Factory, storage.factory_address);
    let feeAddress = factory.fee_addr();
    let feeOn = feeAddress.is_some();
    let _kLast:U128 = U128::from((0,storage.lp_supply)); // gas savings
    let rootK:U128  = (U128::from((0, reserve_0))*U128::from((0, reserve_1))).sqrt();
    if (feeOn) {
        if (_kLast != U128::min()) {
            let rootKLast :U128 = _kLast.sqrt();
            if (rootK > rootKLast) {
                let numerator :U128 = U128::from((0,storage.token_supply))*(rootK-rootKLast);
                let denominator :U128 = rootK*FIVE_U128 +rootKLast;
                let liquidity :u64= (numerator / denominator).as_u64().unwrap();
                if (liquidity > 0) {
                    mint_to(liquidity, feeAddress.unwrap());
                }
            }
        }
    } 
}


impl Swap for Contract {
    // set token
    #[storage(read,write)]
    fn initialize(address_0:ContractId, address_1:ContractId){
        require(storage.token_0_address.is_none(), SwapError::ReinitializeError);
        if address_0.into() < address_1.into(){
            storage.token_0_address = Option::Some(address_0);
            storage.token_1_address = Option::Some(address_1);
        } else{
            storage.token_0_address = Option::Some(address_1);
            storage.token_1_address = Option::Some(address_0);
        }
        
    }

    //TODO: Remove after testing
    #[storage(read), payable]
    fn deposit(){
        require(storage.token_0_address.is_some(), SwapError::NotInitialized);
        let msg_asset = msg_asset_id();
        require(msg_asset == storage.token_0_address.unwrap() || msg_asset == storage.token_1_address.unwrap() || msg_asset == contract_id(), SwapError::InvalidAsset);
    }

    //TODO add implementation or remove function
    #[storage(read, write)]
    fn withdraw(amount: u64, asset_id: ContractId){
        require(storage.token_0_address.is_some(), SwapError::NotInitialized);
    }

    

    #[storage(read)]
    fn quote(amount_0:u64, amount_1: u64) -> u64{
        require(storage.token_0_address.is_some(), SwapError::NotInitialized);
        require(storage.reserve_0 > 0 && storage.reserve_1 > 0, SwapError::PoolLiquidityNotEnough);
        let (input_amount, input_reserve, output_reserve) = if amount_0 != 0 {
            (amount_0, storage.reserve_0, storage.reserve_1)
        }else{
            (amount_1, storage.reserve_1, storage.reserve_0)
        };
        let output_amount = maximum_output_given_exact_input(input_amount, input_reserve, output_reserve);
        output_amount
    }


    #[storage(read,write)]
    fn add_liquidity() -> u64{
        // revert if uninitailized
        let address_0 = storage.token_0_address.unwrap();
        let address_1 = storage.token_1_address.unwrap();
        // TODO: add deadline like require(deadline > height());
        let amount_0 = this_balance(address_0) - storage.reserve_0;
        let amount_1 = this_balance(address_1) - storage.reserve_1;
        require(amount_0 > 0 && amount_1 > 0, SwapError::InputNotEnough);
        //TODO: add error type
        let sender = msg_sender().unwrap();
        let liquidity = storage.lp_supply;
        let mut to_mint: u64 = 0;
        if liquidity > 0{
            // add liquidity to reserve
            let liquidity_0:u64 = multiply_div(amount_0,liquidity,storage.reserve_0);
            let liquidity_1:u64 = multiply_div(amount_1,liquidity,storage.reserve_1);
            to_mint = if liquidity_0 < liquidity_1 {
                liquidity_0
            }else{
                liquidity_1
            };
            mint_to(to_mint, sender);
        }
        
        else {
            // add minimum liquidity to reserve, revert if less than minimum
            to_mint = initial_liquidity(amount_0, amount_1);
            storage.token_supply += MINIMUM_LIQUIDITY.as_u64().unwrap(); // lock minimum liquidity
            mint_to(to_mint, sender);
        }
        // // update parameters
        storage.token_supply += to_mint;
        storage.lp_supply = storage.lp_supply + to_mint;
        storage.reserve_0 = storage.reserve_0 + amount_0;
        storage.reserve_1 = storage.reserve_1 + amount_1;
        to_mint
    }

    // remove Liquidity
    //TODO: make sure contract keep no balance for lp token, perform remove_liquidity with curreent balance.
    #[storage(read,write)]
    fn remove_liquidity() ->(u64,u64){
        require(storage.token_0_address.is_some(), SwapError::NotInitialized);
        let liquidity = storage.lp_supply;
        let to_burn = this_balance(contract_id());

        // positive pool liquidity
        require(to_burn > 0, SwapError::LpTokenNotEnough);
        require(to_burn < liquidity, SwapError::PoolLiquidityNotEnough);
        // require(min_asset_0 > 0 && min_asset_1 > 0, SwapError::PlaceholderError);

        let asset_0_address = storage.token_0_address.unwrap();
        let asset_1_address = storage.token_1_address.unwrap();
        let reserve_0_amount = storage.reserve_0;
        let reserve_1_amount = storage.reserve_1;
        let asset_0_to_remove = multiply_div(to_burn,reserve_0_amount, liquidity);
        let asset_1_to_remove = multiply_div(to_burn,reserve_1_amount, liquidity);

        // amount lower bound
        require(asset_0_to_remove > storage.min_asset_0,SwapError::LiquidityRemovedNotEnough);
        require(asset_1_to_remove > storage.min_asset_1,SwapError::LiquidityRemovedNotEnough);

        // revert if msg_sender not exist
        let sender = msg_sender().unwrap();
        burn(to_burn);

        storage.lp_supply = liquidity - to_burn;
        storage.token_supply -= to_burn;
        storage.reserve_0 = reserve_0_amount - asset_0_to_remove;
        storage.reserve_1 = reserve_1_amount - asset_1_to_remove;

        transfer(asset_0_to_remove, asset_0_address, sender);
        transfer(asset_1_to_remove, asset_1_address, sender);
        (asset_0_to_remove, asset_1_to_remove)
        //TODO: Emit event
    }

    // swap token
    #[storage(read,write)]
    fn swap() -> u64{
        require(storage.token_0_address.is_some(), SwapError::NotInitialized);
        let input_0_amount = this_balance(storage.token_0_address.unwrap()) - storage.reserve_0;
        let input_1_amount = this_balance(storage.token_1_address.unwrap()) - storage.reserve_1;
        require(input_0_amount > 0 || input_1_amount > 0, SwapError::InputNotEnough);

        let token_0_address = storage.token_0_address.unwrap();
        let token_1_address = storage.token_1_address.unwrap();

        let token_0_reserve = storage.reserve_0;
        let token_1_reserve = storage.reserve_1;
        let sender = msg_sender().unwrap();

        //TODO: add error type
        require(input_0_amount < token_0_reserve && input_1_amount < token_1_reserve, SwapError::PoolLiquidityNotEnough);
        if input_0_amount > 0 {
            let output_amount_1 = maximum_output_given_exact_input(input_0_amount, token_0_reserve, token_1_reserve);
            transfer(output_amount_1, token_1_address, sender);
            storage.reserve_0 += input_0_amount;
            storage.reserve_1 -= output_amount_1;
            output_amount_1
        }
        else {
            let output_amount_0 = maximum_output_given_exact_input(input_1_amount, token_1_reserve, token_0_reserve);
            transfer(output_amount_0, token_0_address, sender);
            storage.reserve_0 -= output_amount_0;
            storage.reserve_1 += input_1_amount;
            output_amount_0
        }
    }

    // get token pair
    #[storage(read)]
    fn get_pair() -> (b256,b256) {
        require(storage.token_0_address.is_some(), SwapError::NotInitialized);
        let token_0_id = storage.token_0_address.unwrap();
        let token_1_id = storage.token_1_address.unwrap();
        (token_0_id.into(),token_1_id.into())
    }


    // get token reserve
    #[storage(read)]
    fn get_reserve() -> (u64,u64){
        require(storage.token_0_address.is_some(), SwapError::NotInitialized);
        (storage.reserve_0, storage.reserve_1)
    }

}

#[test]
fn test_maximum_output_given_input(){
    let input_amount:u64 = 100;
    let input_reserve:u64 = 1000;
    let output_reserve:u64 = 2000;
    let expected_result: u64 = 181;
    let result = maximum_output_given_exact_input(input_amount,input_reserve,output_reserve);

    assert(result == expected_result);
}

#[test(should_revert)]
fn test_maximum_output_given_input_revert(){
    let input_amount:u64 = 100;
    let input_reserve:u64 = 1000;
    let output_reserve:u64 = 2000;
    let expected_result: u64 = 181;
    let result = maximum_output_given_exact_input(input_amount,input_reserve,output_reserve);

    assert(result == expected_result+1 || result == expected_result-1);
}


#[test]
fn test_minimum_input_given_exact_output(){
    let output_amount:u64 = 181;
    let input_reserve:u64 = 1000;
    let output_reserve:u64 = 2000;
    let expected_result: u64 =100;
    let result = minimum_input_given_exact_output(output_amount,input_reserve,output_reserve);

    assert(result == expected_result);
}

#[test(should_revert)]
fn test_minimum_input_given_exact_output_revert(){
    let output_amount:u64 = 181;
    let input_reserve:u64 = 1000;
    let output_reserve:u64 = 2000;
    let expected_result: u64 = 100;
    let result = minimum_input_given_exact_output(output_amount,input_reserve,output_reserve);

    assert(result == expected_result+1 || result == expected_result-1);
}



