module test::abi_error_3;

use stylus::error::revert;
use std::ascii::String;

#[ext(abi_error)]
#[allow(unused_field)]
public struct ErrorWithVectors has copy, drop {
    a: vector<u32>,
    b: vector<u128>,
    c: vector<vector<u64>>,
}

#[ext(abi(pure))]
entry fun revert_error_with_vectors(a: vector<u32>, b: vector<u128>, c: vector<vector<u64>>) {
    revert(ErrorWithVectors { a, b, c });
}

public struct NestedStruct(String) has copy, drop;
public struct NestedStruct2 has copy, drop {
    a: String,
    b: u64,
}

#[ext(abi_error)]
#[allow(unused_field)]
public struct ErrorWithNestedStructs has copy, drop {
    a: NestedStruct,
    b: NestedStruct2,
}

#[ext(abi(pure))]
entry fun revert_error_with_nested_structs(a: String, b: String, c: u64) {
    let ns = NestedStruct(a);
    let ns2 = NestedStruct2 { a: b, b: c };
    let error = ErrorWithNestedStructs { a: ns, b: ns2 };
    revert(error);
}

public enum ErrorEnum has drop, copy {
    ERROR_1,
    ERROR_2,
    ERROR_3,    
}

#[ext(abi_error)]
#[allow(unused_field)]
public struct ErrorWithEnum has copy, drop {
    a: ErrorEnum,
    b: vector<ErrorEnum>,
}

entry fun revert_error_with_enum(a: ErrorEnum, b: vector<ErrorEnum>) {
    revert(ErrorWithEnum { a, b });
}