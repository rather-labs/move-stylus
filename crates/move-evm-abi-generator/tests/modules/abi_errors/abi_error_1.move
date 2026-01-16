module test::abi_error_1;

use stylus::error::revert;
use std::ascii::String;

// Standard error type
#[ext(abi_error)]
#[allow(unused_field)]
public struct SimpleError(String) has copy, drop;

#[ext(abi(pure))]
entry fun revert_simple_error(s: String) {
    let error = SimpleError(s);
    revert(error);
}