module test::dynamic_fields_named_id;

use stylus::{object::{Self}, tx_context::TxContext, transfer::{Self}, dynamic_field_named_id as dynamic_field};
use std::ascii::String;

public struct FOO_ has key {}

public struct Foo has key {
    id: object::NamedId<FOO_>,
}

entry fun create_foo() {
    let foo = Foo { id: object::new_named_id<FOO_>() };
    transfer::share_object(foo);
}

entry fun create_foo_owned(ctx: &TxContext) {
    let foo = Foo { id: object::new_named_id<FOO_>() };
    transfer::transfer(foo, ctx.sender());
}

entry fun attach_dynamic_field(foo: &mut Foo, name: String, value: u64) {
    dynamic_field::add(&mut foo.id, name, value);
}

entry fun read_dynamic_field(foo: &Foo, name: String): &u64 {
    dynamic_field::borrow(&foo.id, name)
}

entry fun dynamic_field_exists(foo: &Foo, name: String): bool {
    dynamic_field::exists_(&foo.id, name)
}

entry fun mutate_dynamic_field(foo: &mut Foo, name: String) {
    let val = dynamic_field::borrow_mut(&mut foo.id, name);
    *val = *val + 1;
}

// This test makes sures that two different fields with the same types for key and value get changed
entry fun mutate_dynamic_field_two(foo: &mut Foo, name: String, name_2: String) {
    let val = dynamic_field::borrow_mut(&mut foo.id, name);
    *val = *val + 1;

    let val_2 = dynamic_field::borrow_mut(&mut foo.id, name_2);
    *val_2 = *val_2 + 1;
}

entry fun remove_dynamic_field(foo: &mut Foo, name: String): u64 {
    let value = dynamic_field::remove(&mut foo.id, name);
    value
}

entry fun attach_dynamic_field_addr_u256(foo: &mut Foo, name: address, value: u256) {
    dynamic_field::add(&mut foo.id, name, value);
}

entry fun read_dynamic_field_addr_u256(foo: &Foo, name: address): &u256 {
    dynamic_field::borrow(&foo.id, name)
}

entry fun dynamic_field_exists_addr_u256(foo: &Foo, name: address): bool {
    dynamic_field::exists_(&foo.id, name)
}

entry fun remove_dynamic_field_addr_u256(foo: &mut Foo, name: address): u256 {
    let value = dynamic_field::remove(&mut foo.id, name);
    value
}
