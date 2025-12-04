module hello_world::revert_errors;

use stylus::error::revert;
use std::ascii::String;

public enum MyEnum has drop, copy {
    A,
    B,
    C,
}

// Standard error type
#[ext(abi_error)]
#[allow(unused_field)]
public struct BasicError(String) has copy, drop;

entry fun revert_standard_error(s: String) {
    let error = BasicError(s);
    revert(error);
}

#[ext(abi_error)]
#[allow(unused_field)]
public struct CustomError_ has copy, drop {
    error_message: String,
    error_code: u64,
}
    
entry fun revert_custom_error(s: String, code: u64) {
    revert( CustomError_ { error_message: s, error_code: code });
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
    i: MyEnum,
}

entry fun revert_custom_error2(a: bool, b: u8, c: u16, d: u32, e: u64, f: u128, g: u256, h: address, i: MyEnum) {
    revert(CustomError2 { a, b, c, d, e, f, g, h, i });
}

#[ext(abi_error)]
#[allow(unused_field)]
public struct CustomError3 has copy, drop {
    a: vector<u32>,
    b: vector<u128>,
    c: vector<vector<u64>>,
}

entry fun revert_custom_error3(a: vector<u32>, b: vector<u128>, c: vector<vector<u64>>) {
    revert(CustomError3 { a, b, c });
}

public struct NestedStruct(String) has copy, drop;
public struct NestedStruct2 has copy, drop {
    a: String,
    b: u64,
}

#[ext(abi_error)]
#[allow(unused_field)]
public struct CustomError4 has copy, drop {
    a: NestedStruct,
    b: NestedStruct2,
}

entry fun revert_custom_error4(a: String, b: u64) {
    let error = NestedStruct(a);
    let custom_error = NestedStruct2 { a, b };
    let error = CustomError4 { a: error, b: custom_error };
    revert(error);
}