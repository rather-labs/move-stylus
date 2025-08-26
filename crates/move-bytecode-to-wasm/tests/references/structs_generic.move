module 0x01::structs_generic;

public struct Foo<T> has drop, copy {
    g: T,
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
    bar: Bar<T>,
    baz: Baz,
}

// Static abi sub-struct
public struct Bar<T> has drop, copy {
    g: T,
    a: u16,
    b: u128,
}

// Dynamic abi substruct
public struct Baz has drop, copy {
    a: u16,
    b: vector<u256>,
}

public fun deref_struct(x: Foo<u16>): Foo<u16> {
  let y = &x;
  *y
}

public fun deref_struct_ref(y: &Foo<u16>): Foo<u16> {
  *y
}

public fun identity_struct_ref(x: &Foo<u16>): &Foo<u16> {
    x
}

public fun identity_static_struct_ref(x: &Bar<u16>): &Bar<u16> {
    x
}
public fun call_deref_struct_ref(x: Foo<u16>): Foo<u16> {
    deref_struct_ref(&x)
}

public fun deref_nested_struct(x: Foo<u16>): Foo<u16> {
    let y = &x;
    let z = &*y;
    *z
}

public fun deref_mut_arg(x: &mut Foo<u16>): Foo<u16> {
    *x
}

public fun write_mut_ref(x: &mut Foo<u16>): Foo<u16> {
    x.g = 111;
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
    x.bar.g = 222;
    x.bar.a = 42;
    x.bar.b = 4242;
    x.baz.a = 4242;
    x.baz.b = vector[3];

    *x
}

public fun write_mut_ref_2(x: &mut Foo<u16>): Foo<u16> {
    *x = Foo<u16> {
        g: 111,
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
        bar: Bar { g: 222, a: 42, b: 4242 },
        baz: Baz { a: 4242, b: vector[3] },
    };

    *x
}

public fun freeze_ref(y: Foo<u16>): Foo<u16> {
    let mut x = Foo<u16> {
        g: 111,
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
        bar: Bar { g: 222, a: 42, b: 4242 },
        baz: Baz { a: 4242, b: vector[3] },
    };
    let x_mut_ref: &mut Foo<u16> = &mut x;
    *x_mut_ref = y;
    let x_frozen_ref: &Foo<u16> = freeze(x_mut_ref);
    *x_frozen_ref
}
