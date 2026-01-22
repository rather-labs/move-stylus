module test::dynamic_table;

use stylus::object::{Self};
use stylus::tx_context::TxContext;
use stylus::transfer::{Self};
use stylus::dynamic_field as field;
use std::ascii::{Self};
use std::ascii::String;
use stylus::table::Table;
use stylus::table::{Self};

public struct Foo has key {
    id: object::UID,
}

entry fun create_foo(ctx: &mut TxContext) {
    let foo = Foo { id: object::new(ctx) };
    transfer::share_object(foo);
}

entry fun create_foo_owned(ctx: &mut TxContext) {
    let foo = Foo { id: object::new(ctx) };
    transfer::transfer(foo, ctx.sender());
}

entry fun attach_table(foo: &mut Foo, ctx: &mut TxContext) {
    field::add(
        &mut foo.id,
        ascii::string(b"table"),
        table::new<address, u64>(ctx)
    );
}

entry fun read_table_entry_value(foo: &Foo, key: address): u64 {
    let table = field::borrow<String, Table<address, u64>>(
        &foo.id,
        ascii::string(b"table")
    );
    *table.borrow(key)
}

entry fun create_entry(foo: &mut Foo, key: address, value: u64) {
    let table = field::borrow_mut<String, Table<address, u64>>(
        &mut foo.id,
        ascii::string(b"table")
    );
    table.add(key, value);
}

entry fun contains_entry(foo: &Foo, key: address): bool {
    let table = field::borrow<String, Table<address, u64>>(
        &foo.id,
        ascii::string(b"table")
    );
    table.contains(key)
}

entry fun mutate_table_entry(foo: &mut Foo, key: address) {
    let table = field::borrow_mut<String, Table<address, u64>>(
        &mut foo.id,
        ascii::string(b"table")
    );
    let val = table.borrow_mut(key);
    *val = *val + 1;
}

entry fun mutate_two_entry_values(foo: &mut Foo, key: address, key_2: address) {
    let table = field::borrow_mut<String, Table<address, u64>>(
        &mut foo.id,
        ascii::string(b"table")
    );
    let val = table.borrow_mut(key);
    *val = *val + 1;

    let val_2 = table.borrow_mut(key_2);
    *val_2 = *val_2 + 1;
}

entry fun remove_entry(foo: &mut Foo, key: address): u64 {
    let table = field::borrow_mut<String, Table<address, u64>>(
        &mut foo.id,
        ascii::string(b"table")
    );
    table.remove(key)
}
