module test::dynamic_fields;

use stylus::object as object;
use stylus::tx_context::TxContext;
use stylus::transfer as transfer;
use stylus::dynamic_field as dynamic_field;
use std::ascii::String;

public struct Foo has key {
    id: object::UID,
}

public fun create_foo(ctx: &mut TxContext) {
    let foo = Foo { id: object::new(ctx) };
    transfer::share_object(foo);
}

public fun attach_dynamic_field(foo: &mut Foo, name: String, value: u64) {
    dynamic_field::add(&mut foo.id, name, value);
}

public fun read_dynamic_field(foo: &Foo, name: String): &u64 {
    dynamic_field::borrow(&foo.id, name)
}
