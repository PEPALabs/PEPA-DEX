contract;

use std::{
    address::*,
    assert::assert,
    block::*,
    // chain::auth::*,
    context::*,
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

use swap_abi::Swap;
use factory_abi::*;

pub enum FactoryError {
  UninitailizeError: (), 
  PoolAlreadyExist: (),
  TemplateMismatch: (),
  ReinitailizeError: (),
}


storage {
    template_root: b256 = ZERO_B256, 
    swap_pair: StorageMap<(b256, b256), b256> = StorageMap {},
    swap_address: StorageMap<b256, bool> = StorageMap {},
    fee_addr: Option<Identity> = Option::None,
    fee_addr_setter: Option<Identity> = Option::None,
}


impl Factory for Contract {
    // Initialize factory, set pool code hash for validation
    #[storage(read,write)]
    fn initialize(template_swap_id: b256){
        require(storage.template_root == ZERO_B256, FactoryError::ReinitailizeError);
        let root = bytecode_root(ContractId::from(template_swap_id));
        let sender = msg_sender().unwrap();

        storage.template_root = root;
        storage.fee_addr_setter = Option::Some(sender);
        // skip fee_addr
    }

    // get address for a swap pair 
    #[storage(read)]
    fn get_swap(token_0_address: b256, token_1_address: b256) -> Option<b256> {
        require(storage.template_root != ZERO_B256, FactoryError::UninitailizeError);
        let mut token_0 = token_0_address;
        let mut token_1 = token_1_address;
        // keep order
        if token_1 < token_0 {
            token_0 = token_1_address;
            token_1 = token_0_address;
        }
        // return pool address found
        let swap = storage.swap_pair.get((token_0, token_1));
        if swap.is_none() {
            Option::None
        }else{
            swap
        }
        
    }

    // check if a swap pair exist on address, return Option::None if key not exist
    #[storage(read)]
    fn exist_swap(address: b256) -> bool{
        require(storage.template_root != ZERO_B256, FactoryError::UninitailizeError);
        let addr = storage.swap_address.get(address);
        addr.is_some()
    }

    // create swap for token pair
    #[storage(read, write)]
    fn create_swap(swap_id: b256){
        require(storage.template_root != ZERO_B256, FactoryError::UninitailizeError);
        let root = bytecode_root(ContractId::from(swap_id));
        require(root == storage.template_root, FactoryError::TemplateMismatch);
        // TODO: add to storage first by refering to additional function parameters
        let swap = abi(Swap, swap_id);
        // revert if not initialized
        let (token_0,token_1) = swap.get_pair();

        // pair not exist already
        require(storage.swap_pair.get((token_0,token_1)).is_none(), FactoryError::PoolAlreadyExist);
        storage.swap_pair.insert((token_0, token_1),swap_id);
        storage.swap_address.insert(swap_id,true);
    }

    // return fee address 
    #[storage(read)]
    fn fee_addr() -> Option<Identity>{
        require(storage.fee_addr_setter.is_some(), "Contract not initialized");
        storage.fee_addr
    }

    // set fee to address
    #[storage(read,write)]
    fn set_fee_addr(fee_addr: Identity){
        let sender = msg_sender().unwrap();
        require(storage.fee_addr_setter.is_some(), "Contract not initialized");
        require(sender == storage.fee_addr_setter.unwrap(), "Unauthorized Operation");
        storage.fee_addr = Option::Some(fee_addr);
    }

    // set fee setter address
    #[storage(read,write)]
    fn set_fee_addr_setter(fee_setter_addr: Identity){
        let sender = msg_sender().unwrap();
        require(storage.fee_addr_setter.is_some(), "Contract not initialized");
        require(sender == storage.fee_addr_setter.unwrap(), "Unauthorized Operation");
        storage.fee_addr_setter = Option::Some(fee_setter_addr);
    }
}

