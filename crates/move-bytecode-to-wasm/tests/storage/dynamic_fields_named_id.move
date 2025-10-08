module test::dynamic_fields_named_id;

use stylus::object as object;
use stylus::tx_context::TxContext;
use stylus::transfer as transfer;
use stylus::dynamic_field_named_id as dynamic_field;
use std::ascii::String;

public struct FOO_ has key {}

public struct Foo has key {
    id: object::NamedId<FOO_>,
}

public fun create_foo() {
    let foo = Foo { id: object::new_named_id<FOO_>() };
    transfer::share_object(foo);
}

public fun create_foo_owned(ctx: &mut TxContext) {
    let foo = Foo { id: object::new_named_id<FOO_>() };
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

// This test makes sures that two different fields with the same types for key and value get changed
public fun mutate_dynamic_field_two(foo: &mut Foo, name: String, name_2: String) {
    let val = dynamic_field::borrow_mut(&mut foo.id, name);
    *val = *val + 1;

    let val_2 = dynamic_field::borrow_mut(&mut foo.id, name_2);
    *val_2 = *val_2 + 1;
}

public fun remove_dynamic_field(foo: &mut Foo, name: String): u64 {
    let value = dynamic_field::remove(&mut foo.id, name);
    value
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

public fun remove_dynamic_field_addr_u256(foo: &mut Foo, name: address): u256 {
    let value = dynamic_field::remove(&mut foo.id, name);
    value
}
