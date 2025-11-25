module test::abi_error_2;

use stylus::error::revert;
use std::ascii::String;

#[ext(abi_error)]
#[allow(unused_field)]
public struct CustomError has copy, drop {
    error_message: String,
    error_code: u64,
}
    
entry fun revert_custom_error(s: String, code: u64) {
    revert( CustomError { error_message: s, error_code: code });
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

entry fun revert_custom_error2(a: bool, b: u8, c: u16, d: u32, e: u64, f: u128, g: u256, h: address) {
    revert(CustomError2 { a, b, c, d, e, f, g, h });
}