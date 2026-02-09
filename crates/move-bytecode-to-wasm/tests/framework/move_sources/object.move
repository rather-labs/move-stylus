module test::object;

use stylus::{
    tx_context::TxContext, 
    object::{Self, ID, UID}, 
    transfer::{Self}
};

public struct Foo has key {
    id: UID,
    value: u64
}

entry fun create_frozen_foo(ctx: &mut TxContext) {
    let foo = Foo {
        id: object::new(ctx),
        value: 101,
    };
    transfer::freeze_object(foo);
}

entry fun create_shared_foo(ctx: &mut TxContext) {
    let foo = Foo {
        id: object::new(ctx),
        value: 101,
    };
    transfer::share_object(foo);
}

entry fun create_owned_foo(ctx: &mut TxContext) {
    let foo = Foo {
        id: object::new(ctx),
        value: 101,
    };
    transfer::transfer(foo, ctx.sender());
}

entry fun get_foo_id(foo: &Foo): ID {
    object::id(foo)
}

entry fun get_foo_id_ref(foo: &Foo): &ID {
    object::borrow_id(foo)
}

entry fun get_foo_id_address(foo: &Foo): address {
    object::id_address(foo)
}