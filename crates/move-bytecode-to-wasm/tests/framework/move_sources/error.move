module test::error;

use stylus::error::revert;
use std::ascii::String;

// Standard error type
#[ext(abi_error)]
#[allow(unused_field)]
public struct SimpleError(String) has copy, drop;

entry fun revert_standard_error(s: String) {
    let error = SimpleError(s);
    revert(error);
}

#[ext(abi_error)]
#[allow(unused_field)]
public struct CustomError has copy, drop {
    error_message: String,
    error_code: u64,
}

entry fun revert_custom_error(s: String, code: u64) {
    revert(CustomError { error_message: s, error_code: code });
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
    let error = CustomError2 { a, b, c, d, e, f, g, h };
    revert(error);
}

#[ext(abi_error)]
#[allow(unused_field)]
public struct CustomError3 has copy, drop {
    a: vector<u32>,
    b: vector<u128>,
    c: vector<vector<u64>>,
}

entry fun revert_custom_error3(a: vector<u32>, b: vector<u128>, c: vector<vector<u64>>) {
    let error = CustomError3 { a, b, c };
    revert(error);
}

public struct NestedStruct1(String) has copy, drop;

public struct NestedStruct2 has copy, drop {
    a: String,
    b: u64,
}

#[ext(abi_error)]
#[allow(unused_field)]
public struct CustomError4 has copy, drop {
    a: NestedStruct1,
    b: NestedStruct2,
}

entry fun revert_custom_error4(a: String, b: String, c: u64) {
    let error = NestedStruct1(a);
    let custom_error = NestedStruct2 { a: b, b: c };
    let error = CustomError4 { a: error, b: custom_error };
    revert(error);
}

#[error]
const ETest: vector<u8> = b"Error for testing #[error] macro";

entry fun abort_with_clever_error() {
    abort(ETest)
}

#[error]
const EAnotherTest: vector<u8> = b"Another error for testing clever errors";

entry fun abort_with_another_clever_error() {
    abort(EAnotherTest)
}
