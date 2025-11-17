module test::simple;

// Standard error type
#[allow(unused_field)]
public struct S {
    a: u64,
    b: u256
}

entry fun f(a: u64, b: u256): S {
   S { a, b }
}