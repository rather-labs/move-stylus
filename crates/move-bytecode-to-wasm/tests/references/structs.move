module 0x01::structs;

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

public fun deref_struct(x: Foo): Foo {
  let y = &x;
  *y
}

public fun deref_struct_ref(y: &Foo): Foo {
  *y
}

public fun call_deref_struct_ref(x: Foo): Foo {
    deref_struct_ref(&x)
}

public fun deref_nested_struct(x: Foo): Foo {
    let y = &x;
    let z = &*y;
    *z
}

public fun deref_mut_arg(x: &mut Foo): Foo {
    *x
}

public fun write_mut_ref(x: &mut Foo): Foo {
    x.q = @0xdeadbeef;
    x.r = vector[0, 3, 0, 3, 4, 5, 6];
    x.s = vector[6, 5, 4, 3, 0, 3, 0];
    x.t = false;
    x.u = 42;
    x.v = 4242;
    x.w = 424242;
    x.x = 42424242;
    x.y = 4242424242;
    x.z = 424242424242;
    x.bar.a = 42;
    x.bar.b = 4242;
    x.baz.a = 4242;
    x.baz.b = vector[3];

    *x
}

public fun write_mut_ref_2(x: &mut Foo): Foo {
    *x = Foo {
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

    *x
}

public fun freeze_ref(y: Foo): Foo {
    let mut x = Foo {
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
    let x_mut_ref: &mut Foo = &mut x;
    *x_mut_ref = y;
    let x_frozen_ref: &Foo = freeze(x_mut_ref);
    *x_frozen_ref
}
