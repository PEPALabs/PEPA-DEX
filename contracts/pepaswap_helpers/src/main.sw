library pepaswap_helpers;

use std::{
    address::*,
    block::*,
    // chain::auth::*,
    context::{*},
    result::*,
    storage::*,
    revert::revert,
    identity::Identity,
    call_frames::*,
    auth::*,
    u128::U128,
    constants::ZERO_B256,
};

pub const NOMINATOR = U128::from((0,9997));
pub const DENOMINATOR = U128::from((0,10000));

#[storage(read,write)]
pub fn get_b256(key: b256) -> b256 {
    get::<b256>(key).unwrap()
}


// Store b256 values on memory
#[storage(read,write)]
pub fn store_b256(key: b256, value: b256) {
    store(key,value);
}

/// Return the sender as an Address or panic
pub fn get_msg_sender_address_or_panic() -> Address {
    let sender: Result<Identity, AuthError> = msg_sender();
    if let Identity::Address(address) = sender.unwrap() {
       address
    } else {
       revert(0);
    }
}

pub fn multiply_div(a: u64, b: u64, c: u64) -> u64 {
   let calculation = (U128::from((0, a)) * U128::from((0, b)));
   let result_wrapped = (calculation / U128::from((0, c))).as_u64();

   // TODO remove workaround once https://github.com/FuelLabs/sway/pull/1671 lands.
   match result_wrapped {
       Result::Ok(inner_value) => inner_value, _ => revert(0),
   }
}
// initial version
// determine required input amount for swapping out @output_amount of another token in a pool
pub fn minimum_input_given_exact_output(
   output_amount: u64,
   input_reserve: u64,
   output_reserve: u64,
) -> u64 {
   assert(input_reserve > 0 && output_reserve > 0);
   let numerator = U128::from((0,input_reserve)) * U128::from((0,output_amount));
   let denominator = U128::from((0,output_reserve)) - U128::from((0,output_amount));
   let result_wrapped = (numerator / denominator).as_u64();
   if denominator > numerator{
       0 // round down float points
   }else {
       result_wrapped.unwrap() + 1 // add 1 since rounded
   }
}

// determine output given exact input amount
pub fn maximum_output_given_exact_input(
   input_amount: u64,
   input_reserve: u64,
   output_reserve: u64,
) -> u64 {
   assert(input_reserve > 0 && output_reserve > 0);
   let numerator = U128::from((0,input_amount)) * U128::from((0,output_reserve));
   let denominator = U128::from((0,input_reserve)) + U128::from((0,input_amount));
   let result_wrapped = (numerator / denominator).as_u64();
   result_wrapped.unwrap()
}

// with fee 0.03% on
// determine required input amount for swapping out @output_amount of another token in a pool
pub fn minimum_input_given_exact_output_with_fee(
   output_amount: u64,
   input_reserve: u64,
   output_reserve: u64,
) -> u64 {
   assert(input_reserve > 0 && output_reserve > 0);
   let numerator = U128::from((0,input_reserve)) * U128::from((0,output_amount));
   let denominator = U128::from((0,output_reserve)) - U128::from((0,output_amount));
   let result_wrapped = ((numerator*NOMINATOR) / (denominator*DENOMINATOR)).as_u64();
   if denominator > numerator{
       0 // round down float points
   }else {
       result_wrapped.unwrap() + 1 // add 1 since rounded
   }
}

// With fee 0.03% on
pub fn maximum_output_given_exact_input_with_fee(
   input_amount: u64,
   input_reserve: u64,
   output_reserve: u64,
) -> u64 {
   assert(input_reserve > 0 && output_reserve > 0);
   let numerator = U128::from((0,input_amount)) * U128::from((0,output_reserve));
   let denominator = U128::from((0,input_reserve)) + U128::from((0,input_amount));
   let result_wrapped = ((numerator*NOMINATOR) / (denominator*DENOMINATOR)).as_u64();
   result_wrapped.unwrap()
}