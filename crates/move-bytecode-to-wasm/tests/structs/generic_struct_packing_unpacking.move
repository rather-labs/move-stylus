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


// Static abi struct
public struct Bar has drop {
    g: u32,
    q: address,
}

public fun test2(foo: Bar): (u32, address) {
    (
        foo.g,
        foo.q,
    )
}
