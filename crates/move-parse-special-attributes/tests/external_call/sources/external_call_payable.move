module test::external_call_payable;

#[ext(external_call)]
public struct ExternalCall has drop {
    contract_address: address,
    delegate: bool,
}

#[ext(external_call, payable)]
public native fun external_payable_function(
    self: &ExternalCall,
    value: u256,
    amount: u64
): u64;

#[ext(external_call, payable)]
public native fun external_payable_function_no_value(
    self: &ExternalCall,
    amount: u64
): u64;

#[ext(external_call, payable)]
public native fun external_payable_function_no_value_2(
    self: &ExternalCall,
): u64;

#[ext(external_call, payable)]
public native fun external_payable_function_wrong_value_type(
    self: &ExternalCall,
    value: u128,
    amount: u64
): u64;
