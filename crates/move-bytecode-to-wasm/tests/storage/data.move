//! This module test the save and retrieval of different data structures in storage.
module test::data;

use stylus::transfer as transfer;
use stylus::object as object;
use stylus::object::UID;
use stylus::tx_context::TxContext;
use stylus::tx_context as tx_context;

public struct Foo has key {
    id: UID,
    value: u64
}

public struct FooBar has key {
    id: UID,
    value: u64,
    bar: Bar,
}

public struct Bar has store {
    a: u64,
    b: address
}

public fun create_foo(ctx: &mut TxContext, value: u64) {
    transfer::share_object(Foo {
        id: object::new(ctx),
        value
    });
}

public fun get_foo(foo: Foo): Foo {
    foo
}

public fun create_foo_bar(ctx: &mut TxContext, value: u64, a: u64, b: address) {
    transfer::share_object(FooBar {
        id: object::new(ctx),
        value,
        bar: Bar { a, b }
    });
}

public fun get_foo_bar(foo_bar: FooBar): FooBar {
    foo_bar
}
