module test::error;

use stylus::error::revert;
use std::ascii::String;

// Standard error type
#[allow(unused_field)]
public struct Error(String) has copy, drop;

entry fun revert_standard_error(error: Error) {
    revert(error);
}

#[allow(unused_field)]
public struct CustomError has copy, drop {
    message: String,
    addr: address,
    boolean: bool,
    second_message: String,
}

entry fun revert_custom_error(error: CustomError) {
    revert(error);
}

#[allow(unused_field)]
public struct CustomError2 has copy, drop {
    a: u16,
    b: u32,
    c: u64,
    message: String,
    d: u128,
    e: u256,
}

entry fun revert_custom_error2(error: CustomError2) {
    revert(error);
}

#[allow(unused_field)]
public struct CustomError3 has copy, drop {
    a: u16,
    b: u32,
    c: u64,
    d: u128,
    e: u256,
}

entry fun revert_custom_error3(error: CustomError3) {
    revert(error);
}

#[allow(unused_field)]
public struct CustomError4 has copy, drop {
    a: u16,
    b: CustomError3,
    c: u32,
}

entry fun revert_custom_error4(error: CustomError4) {
    revert(error);
}

#[allow(unused_field)]
public struct CustomError5 has copy, drop {
    a: vector<u32>,
    b: vector<u128>,
}

entry fun revert_custom_error5(error: CustomError5) {
    revert(error);
}