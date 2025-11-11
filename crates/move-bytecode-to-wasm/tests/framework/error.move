module test::error;

use stylus::error::revert;
use std::ascii::String;

// Standard error type
#[ext(abi_error)]
#[allow(unused_field)]
public struct Error(String) has copy, drop;

entry fun revert_standard_error(error: Error) {
    revert(error);
}

#[ext(abi_error)]
#[allow(unused_field)]
public struct CustomError has copy, drop {
    error_message: String,
    error_code: u64,
}

entry fun revert_custom_error(error: CustomError) {
    revert(error);
}

#[allow(unused_field)]
#[ext(abi_error)]
public struct CustomError2 has copy, drop {
    a: bool,
    b: u8,
    c: u16,
    d: u32,
    e: u64,
    f: u128,
    g: u256,
    h: address,
}

entry fun revert_custom_error2(error: CustomError2) {
    revert(error);
}

#[ext(abi_error)]
#[allow(unused_field)]
public struct CustomError3 has copy, drop {
    a: vector<u32>,
    b: vector<u128>,
    c: vector<vector<u64>>,
}

entry fun revert_custom_error3(error: CustomError3) {
    revert(error);
}

#[ext(abi_error)]
#[allow(unused_field)]
public struct CustomError4 has copy, drop {
    a: CustomError,
    b: CustomError2,
}

entry fun revert_custom_error4(error: CustomError4) {
    revert(error);
}