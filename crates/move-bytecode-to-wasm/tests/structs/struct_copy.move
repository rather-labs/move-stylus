module 0x00::struct_copy;

public struct Foo has drop, copy {
    q: address,
    r: vector<u32>,
    s: vector<u128>,
    t: bool,
    u: u8,
    v: u16,
    w: u32,
    x: u64,
    y: u128,
    z: u256,
    bar: Bar,
    baz: Baz,
}

// Static abi sub-struct
public struct Bar has drop, copy {
    a: u16,
    b: u128,
}

// Dynamic abi substruct
public struct Baz has drop, copy {
    a: u16,
    b: vector<u256>,
}

public fun structCopy(foo: Foo): (Foo, Foo) {
    let mut foo_2 = foo;

    foo_2.q = @0xdeadbeef;
    foo_2.r = vector[0, 3, 0, 3, 4, 5, 6];
    foo_2.s = vector[6, 5, 4, 3, 0, 3, 0];
    foo_2.t = false;
    foo_2.u = 42;
    foo_2.v = 4242;
    foo_2.w = 424242;
    foo_2.x = 42424242;
    foo_2.y = 4242424242;
    foo_2.z = 424242424242;
    foo_2.bar.a = 42;
    foo_2.bar.b = 4242;
    foo_2.baz.a = 4242;
    foo_2.baz.b = vector[3];

    (foo, foo_2)
}

public fun structCopy2(): (Foo, Foo) {
    let foo_1 = Foo {
        q: @0xdeadbeef,
        r : vector[0, 3, 0, 3, 4, 5, 6],
        s : vector[6, 5, 4, 3, 0, 3, 0],
        t : false,
        u : 42,
        v : 4242,
        w : 424242,
        x : 42424242,
        y : 4242424242,
        z : 424242424242,
        bar: Bar { a: 42, b: 4242 },
        baz: Baz { a: 4242, b: vector[3] },
    };

    let foo_2 = foo_1;
    (foo_1, foo_2)
}
