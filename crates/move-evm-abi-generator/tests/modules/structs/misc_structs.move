module test::misc_structs;

public struct NestedStruct<phantom T, phantom U> has copy, drop {
    a: bool,
    b: vector<bool>,
}

public struct GenericStruct<T, U> has copy, drop {
    a: T,
    b: U,
    c: GenericEnum<T, U>,
    d: SimpleEnum,
    e: NestedStruct<T, U>,
}

public enum GenericEnum<phantom T, phantom U> has copy, drop {
    A,
    B,
}

public enum SimpleEnum has copy, drop {
    A,
    B,
}

entry fun test_misc(a: u32, b: u128, c: GenericEnum<u32, u128>, d: SimpleEnum, e: NestedStruct<u32, u128>): GenericStruct<u32, u128> {
    let s = GenericStruct { a, b, c, d, e };
    s
}