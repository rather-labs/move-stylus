module test::peep;

use stylus::peep as stylus_peep;
use stylus::object::UID;
use stylus::object as object;
use stylus::tx_context::TxContext;
use stylus::transfer::transfer;

public struct Foo has key, store {
    id: UID,
    secret: u32
}

public struct Bar has key {
    id: UID,
    a: vector<u128>,
    b: Foo
}

entry fun create_owned_foo(owner_address: address, ctx: &mut TxContext) {
    let foo = Foo {
        id: object::new(ctx),
        secret: 100
    };

    transfer(foo, owner_address);
}

entry fun create_owned_bar(owner_address: address, ctx: &mut TxContext) {
    let bar = Bar {
        id: object::new(ctx),
        a: vector[1, 2, 3],
        b: Foo {
            id: object::new(ctx),
            secret: 100
        }
    };
    transfer(bar, owner_address);
}

entry fun owner_peep_foo(foo: &Foo, ctx: &TxContext): u32 {
    let foo_: &Foo = stylus_peep::peep<Foo>(ctx.sender(), &foo.id);
    foo_.secret
}

entry fun peep_foo(owner: address, foo_id: &UID): &Foo {
    stylus_peep::peep<Foo>(owner, foo_id)
}

entry fun peep_bar(owner: address, bar_id: &UID): &Bar {
    stylus_peep::peep<Bar>(owner, bar_id)
}

entry fun call_indirect_peep_foo(owner: address, foo_id: &UID, ctx: &mut TxContext): u32 {
    let foo_: &Foo = peep_foo(owner, foo_id);
    create_owned_foo(owner, ctx);
    foo_.secret
}
