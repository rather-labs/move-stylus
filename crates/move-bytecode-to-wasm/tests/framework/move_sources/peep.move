module test::peep;

use stylus::peep as stylus_peep;
use stylus::object::UID;
use stylus::object as object;
use stylus::tx_context::TxContext;
use stylus::transfer::transfer;

public struct Foo has key {
    id: UID,
    secret: u32
}

entry fun create_owned_foo(owner_address: address, ctx: &mut TxContext) {
    let foo = Foo {
        id: object::new(ctx),
        secret: 100
    };

    transfer(foo, owner_address);
}

entry fun owner_peep_foo(foo: &Foo, ctx: &TxContext): u32 {
    let foo_: &Foo = stylus_peep::peep<Foo>(ctx.sender(), &foo.id);
    foo_.secret
}

entry fun peep_foo(owner: address,foo_id: &UID, ctx: &TxContext): &Foo {
    stylus_peep::peep<Foo>(owner, foo_id)
}