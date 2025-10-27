module test::external_call;

use stylus::contract_calls::{ContractCallResult, ContractCallEmptyResult};

#[ext(external_call)]
public struct ExternalCall(CrossContractCall) has drop;


#[ext(external_call)]
public native fun external_function_ok_1(
    self: &ExternalCall,
    amount: u64
): ContractCallResult<u64>;

#[ext(external_call)]
public native fun external_function_ok_2(
    self: &ExternalCall,
    amount: u64
): ContractCallEmptyResult;

#[ext(external_call)]
public native fun external_call_invalid_return(
    self: &ExternalCall,
    amount: u64
): u64;

#[ext(external_call)]
public fun external_function_not_native(
    self: &ExternalCall,
    amount: u64
): ContractCallResult<u64> { 1 }
