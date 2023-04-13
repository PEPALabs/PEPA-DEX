library swap_abi;

use std::{
    address::*,
    assert::assert,
    block::*,
    // chain::auth::*,
    contract_id::ContractId,
    result::*,
    revert::revert,
    storage::*,
    token::*,
    auth::*,
    call_frames::{contract_id, msg_asset_id},
};
// TODO: add 
abi Swap {
    // initialize and set token address pair 
    #[storage(read,write)]
    fn initialize(address_0:ContractId, address_1:ContractId);

    // helper function to deposit liquidity
    #[storage(read),payable]
    fn deposit();

    // withdraw liquidity
    #[storage(read, write)]
    fn withdraw(amount: u64, asset_id: ContractId);

    // quote exchange rate
    #[storage(read)]
    fn quote(amount_0:u64, amount_1:u64)->u64;

    // Add Liquidity
    #[storage(read,write)]
    fn add_liquidity()-> u64;

    // remove Liquidity
    #[storage(read,write)]
    fn remove_liquidity()-> (u64,u64);

    // swap exact token for another token
    // @return:
    #[storage(read,write)]
    fn swap()->u64;

    // get token pair
    #[storage(read)]
    fn get_pair() -> (b256,b256);

    // get token reserve
    #[storage(read)]
    fn get_reserve() -> (u64,u64);
}

