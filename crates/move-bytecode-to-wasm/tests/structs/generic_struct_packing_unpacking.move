module 0x00::generic_struct_packing_unpacking;

// Static abi struct
public struct Foo<T> has drop {
    g: T,
    q: address,
}

public fun test(foo: Foo<u32>): (u32, address) {
    (
        foo.g,
        foo.q,
    )
}

