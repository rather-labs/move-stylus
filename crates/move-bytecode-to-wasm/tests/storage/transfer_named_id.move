module test::transfer_named_id;

use stylus::object as object;
use stylus::object::NamedId;
use stylus::transfer as transfer;

// ============================================================================
// STRUCT DEFINITIONS
// ============================================================================

public struct FOO_ has key {}
public struct BAR_ has key {}
public struct BAZ_ has key {}
public struct BEZ_ has key {}
public struct BIZ_ has key {}

// Simple struct with a single value field
public struct Foo has key {
    id: NamedId<FOO_>,
    value: u64
}

// Struct with a vector field
public struct Bar has key {
    id: NamedId<BAR_>,
    a: u64,
    c: vector<u64>
}

// Simple value struct (no key)
public struct Qux has store, drop {
    a: u64,
    b: u128,
    c: u128
}

// Struct with nested field struct
public struct Baz has key {
    id: NamedId<BAZ_>,
    a: u64,
    c: Qux
}

// Complex struct with nested vectors
public struct Bez has key {
    id: NamedId<BEZ_>,
    a: u64,
    c: vector<Qux>,
    d: vector<vector<u128>>,
    e: u8
}

// Generic value struct
public struct Quz<T> has store, drop {
    a: T,
    b: u128,
    c: u128
}

// Generic struct with key and nested field struct
public struct Biz<T: copy> has key {
    id: NamedId<BIZ_>,
    a: T,
    b: Quz<T>,
    c: vector<Quz<T>>,
}

// ============================================================================
// FOO FUNCTIONS
// ============================================================================

public fun create_shared() {
    let foo = Foo {
        id: object::new_named_id<FOO_>(),
        value: 101,
    };
    transfer::share_object(foo);
}

public fun create_owned(recipient: address) {
    let foo = Foo {
        id: object::new_named_id<FOO_>(),
        value: 101,
    };
    transfer::transfer(foo, recipient);
}

public fun create_frozen() {
    let foo = Foo {
        id: object::new_named_id<FOO_>(),
        value: 101,
    };
    transfer::freeze_object(foo);
}

public fun read_value(foo: &Foo): u64 {
    foo.value
}

public fun set_value(foo: &mut Foo, value: u64) {
    foo.value = value;
}

public fun increment_value(foo: &mut Foo) {
    foo.value = foo.value + 1;
}

public fun get_foo(foo: &Foo): &Foo {
    foo
}

public fun delete_obj(foo: Foo) {
    let Foo { id, value: _ } = foo;
    id.remove();
}

fun freeze_obj(foo: Foo) {
    transfer::freeze_object(foo);
}

public fun share_obj(foo: Foo) {
    transfer::share_object(foo);
}

public fun transfer_obj(foo: Foo, recipient: address) {
    transfer::transfer(foo, recipient);
}

// ============================================================================
// BAR FUNCTIONS
// ============================================================================

public fun create_bar() {
    let bar = Bar {
        id: object::new_named_id<BAR_>(),
        a: 101,
        c: vector[1, 2, 3, 4, 5, 6, 7, 8, 9],
    };
    transfer::share_object(bar);
}

public fun get_bar(bar: &Bar): &Bar {
    bar
}

public fun delete_bar(bar: Bar) {
    let Bar { id, a: _, c: _ } = bar;
    id.remove();
}

// ============================================================================
// BAZ FUNCTIONS
// ============================================================================

public fun create_baz(recipient: address, share: bool) {
    let baz = Baz {
        id: object::new_named_id<BAZ_>(),
        a: 101,
        c: Qux { a: 42, b: 55, c: 66 },
    };

    if (share) {
        transfer::share_object(baz);
    } else {
        transfer::transfer(baz, recipient);
    }
}

public fun get_baz(baz: &Baz): &Baz {
    baz
}

public fun delete_baz(baz: Baz) {
    let Baz { id, a: _, c: _ } = baz;
    id.remove();
}

// ============================================================================
// BEZ FUNCTIONS
// ============================================================================

public fun create_bez() {
    let bez = Bez {
        id: object::new_named_id<BEZ_>(),
        a: 101,
        c: vector[
            Qux { a: 42, b: 55, c: 66 },
            Qux { a: 43, b: 56, c: 67 },
            Qux { a: 44, b: 57, c: 68 }
        ],
        d: vector[
            vector[1, 2, 3],
            vector[4],
            vector[],
            vector[5, 6]
        ],
        e: 17,
    };
    transfer::share_object(bez);
}

public fun get_bez(bez: &Bez): &Bez {
    bez
}

public fun delete_bez(bez: Bez) {
    let Bez { id, a: _, c: _, d: _, e: _ } = bez;
    id.remove();
}

// ============================================================================
// BIZ FUNCTIONS
// ============================================================================

public fun create_biz() {
    let biz = Biz<u64> {
        id: object::new_named_id<BIZ_>(),
        a: 101,
        b: Quz<u64> { a: 42, b: 55, c: 66 },
        c: vector[
            Quz<u64>{ a: 42, b: 55, c: 66 },
            Quz<u64>{ a: 43, b: 56, c: 67 },
            Quz<u64>{ a: 44, b: 57, c: 68 },
        ],
    };
    transfer::share_object(biz);
}

public fun get_biz(biz: &Biz<u64>): &Biz<u64> {
    biz
}

public fun delete_biz(biz: Biz<u64>) {
    let Biz { id, a: _, b: _, c: _ } = biz;
    id.remove();
}
