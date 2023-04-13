library factory_abi;

use std::{
    address::*,
    assert::assert,
    block::*,
    // chain::auth::*,
    // context::{call_frames::*,*},
    contract_id::ContractId,
    constants::*,
    external::bytecode_root,
    hash::*,
    result::*,
    revert::revert,
    storage::*,
    token::*,
    u128::U128,
    //comment out forc 0.33.1 import
    auth::*,
    call_frames::{contract_id, msg_asset_id},
};


abi Factory {

    // Initialize factory
    #[storage(read,write)]
    fn initialize(template_swap_id: b256);

    // get address for a swap pair 
    #[storage(read)]
    fn get_swap(token_0_address: b256, token_1_address: b256) -> Option<b256> ;

    // check if a swap pair exist on address
    #[storage(read)]
    fn exist_swap(address: b256) -> bool;

    // create swap for token pair
    #[storage(read,write)]
    fn create_swap(swap_id: b256);

    // return fee address, option none represents zero fee
    #[storage(read)]
    fn fee_addr()->Option<Identity>;

    // set fee to address
    #[storage(read,write)]
    fn set_fee_addr(fee_addr: Identity);

    // set fee setter address
    #[storage(read,write)]
    fn set_fee_addr_setter(fee_setter_addr: Identity);
}

