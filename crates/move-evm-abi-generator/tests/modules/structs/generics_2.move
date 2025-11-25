module test::generics_2;

public struct GenericStruct<T, U> has copy, drop {
    a: T,
    b: U,
}

entry fun unpack_1(s: GenericStruct<u32, u64>): (u32, u64) {
    (s.a, s.b)
}

entry fun unpack_2(s: GenericStruct<u128, u256>): (u128, u256) {
    (s.a, s.b)
}

entry fun test_generic_structs(s1: GenericStruct<u32, u64>, s2: GenericStruct<u128, u256>): (u32, u64, u128, u256) {
    let (a, b) = unpack_1(s1);
    let (c, d) = unpack_2(s2);
    (a, b, c, d)
}