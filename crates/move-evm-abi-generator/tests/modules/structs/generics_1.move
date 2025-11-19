module test::generics_1;

public struct GenericStruct<T, U> has copy, drop {
    a: T,
    b: U,
}

entry fun test_generic_structs(a1: u32, a2: u64, b1: u128, b2: u256): (GenericStruct<u32, u64>, GenericStruct<u128, u256>) {
    let s1 = GenericStruct<u32, u64> { a: a1, b: a2 };
    let s2 = GenericStruct<u128, u256> { a: b1, b: b2 };
    (s1, s2)
}