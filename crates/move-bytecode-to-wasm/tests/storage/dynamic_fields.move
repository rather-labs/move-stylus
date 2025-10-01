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

public fun create_foo_owned(ctx: &mut TxContext) {
    let foo = Foo { id: object::new(ctx) };
    transfer::transfer(foo, ctx.sender());
}

public fun attach_dynamic_field(foo: &mut Foo, name: String, value: u64) {
    dynamic_field::add(&mut foo.id, name, value);
}

public fun read_dynamic_field(foo: &Foo, name: String): &u64 {
    dynamic_field::borrow(&foo.id, name)
}

public fun dynamic_field_exists(foo: &Foo, name: String): bool {
    dynamic_field::exists_(&foo.id, name)
}

public fun mutate_dynamic_field(foo: &mut Foo, name: String) {
    let val = dynamic_field::borrow_mut(&mut foo.id, name);
    *val = *val + 1;
}

public fun attach_dynamic_field_addr_u256(foo: &mut Foo, name: address, value: u256) {
    dynamic_field::add(&mut foo.id, name, value);
}

public fun read_dynamic_field_addr_u256(foo: &Foo, name: address): &u256 {
    dynamic_field::borrow(&foo.id, name)
}

public fun dynamic_field_exists_addr_u256(foo: &Foo, name: address): bool {
    dynamic_field::exists_(&foo.id, name)
}
