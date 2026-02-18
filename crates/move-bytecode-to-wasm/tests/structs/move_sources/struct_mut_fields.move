module 0x00::struct_mut_fields;

public struct Bar has drop {
    n: u32,
    o: u128,
}

public struct Foo has drop {
    p: Bar,
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
}

entry fun echo_mut_bool(a: bool): bool {
    let mut foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: false,
        u: 2,
        v: 3,
        w: 4,
        x: 5,
        y: 6,
        z: 7,
    };

    foo.t = a;
    foo.t
}

entry fun echo_mut_u8(a: u8): u8 {
    let mut foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: false,
        u: 2,
        v: 3,
        w: 4,
        x: 5,
        y: 6,
        z: 7,
    };

    foo.u = a;
    foo.u
}

entry fun echo_mut_u16(a: u16): u16 {
    let mut foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: false,
        u: 2,
        v: 3,
        w: 4,
        x: 5,
        y: 6,
        z: 7,
    };

    foo.v = a;
    foo.v
}

entry fun echo_mut_u32(a: u32): u32 {
    let mut foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: false,
        u: 2,
        v: 3,
        w: 4,
        x: 5,
        y: 6,
        z: 7,
    };

    foo.w = a;
    foo.w
}

entry fun echo_mut_u64(a: u64): u64 {
    let mut foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: false,
        u: 2,
        v: 3,
        w: 4,
        x: 5,
        y: 6,
        z: 7,
    };

    foo.x = a;
    foo.x
}

entry fun echo_mut_u128(a: u128): u128 {
    let mut foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: false,
        u: 2,
        v: 3,
        w: 4,
        x: 5,
        y: 6,
        z: 7,
    };

    foo.y = a;
    foo.y
}

entry fun echo_mut_u256(a: u256): u256{
    let mut foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: false,
        u: 2,
        v: 3,
        w: 4,
        x: 5,
        y: 6,
        z: 7,
    };

    foo.z = a;
    foo.z
}

entry fun echo_mut_vec_stack_type(a: vector<u32>): vector<u32> {
    let mut foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: false,
        u: 2,
        v: 3,
        w: 4,
        x: 5,
        y: 6,
        z: 7,
    };

    foo.r = a;
    foo.r
}

entry fun echo_mut_vec_heap_type(a: vector<u128>): vector<u128> {
    let mut foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: false,
        u: 2,
        v: 3,
        w: 4,
        x: 5,
        y: 6,
        z: 7,
    };

    foo.s = a;
    foo.s
}

entry fun echo_mut_address(a: address): address {
    let mut foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: false,
        u: 2,
        v: 3,
        w: 4,
        x: 5,
        y: 6,
        z: 7,
    };

    foo.q = a;
    foo.q
}

entry fun echo_bar_struct_fields(a: u32, b: u128): (u32, u128) {
    let mut foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    };

    foo.p.n = a;
    foo.p.o = b;

    (foo.p.n, foo.p.o)
}

fun create_foo(): Foo {
    Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[42],
        s: vector[43],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    }
}

fun mutate_stack_vec(foo: &mut Foo, r: vector<u32>) {
    foo.r = r;
}

entry fun test_mutate_stack_vec(r: vector<u32>): vector<u32> {
    let mut foo = create_foo();
    mutate_stack_vec(&mut foo, r);
    foo.r
}

fun mutate_heap_vec(foo: &mut Foo, s: vector<u128>) {
    foo.s = s;
}

entry fun test_mutate_heap_vec(s: vector<u128>): vector<u128> {
    let mut foo = create_foo();
    mutate_heap_vec(&mut foo, s);
    foo.s
}

fun deref_and_replace_foo(foo: &mut Foo) {
    let foo_ = create_foo();
    *foo = foo_;
}

// The idea of this test is to create a Foo struct, mutate some of it fields, and then call `deref_and_replace_foo` 
// which dereferece the &mut Foo argument and replaces it with a new Foo struct.
// If everything works as expected, we should end up with the same Foo that `create_foo` returns, not the mutated one.
entry fun test_deref_and_replace_foo(): (u8, u16) {
    let mut foo = create_foo();
    foo.u = 11; // Replace the u8 to test that the deref_and_replace_foo function works
    foo.v = 12; // Replace the u16 to test that the deref_and_replace_foo function works
    deref_and_replace_foo(&mut foo); // This should set us back to the original vector
    (foo.u, foo.v)
}