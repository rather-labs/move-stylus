module test::dynamic_table;

use stylus::object as object;
use stylus::tx_context::TxContext;
use stylus::transfer as transfer;
use stylus::dynamic_field as field;
use std::ascii as ascii;
use std::ascii::String;
use stylus::table::Table;
use stylus::table as table;

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

public fun attach_table(foo: &mut Foo, ctx: &mut TxContext) {
    field::add(
        &mut foo.id,
        ascii::string(b"table"),
        table::new<address, u64>(ctx)
    );
}

public fun read_table_entry_value(foo: &Foo, key: address): u64 {
    let table = field::borrow<String, Table<address, u64>>(
        &foo.id,
        ascii::string(b"table")
    );
    *table.borrow(key)
}

public fun create_entry(foo: &mut Foo, key: address, value: u64) {
    let table = field::borrow_mut<String, Table<address, u64>>(
        &mut foo.id,
        ascii::string(b"table")
    );
    table.add(key, value);
}

public fun contains_entry(foo: &Foo, key: address): bool {
    let table = field::borrow<String, Table<address, u64>>(
        &foo.id,
        ascii::string(b"table")
    );
    table.contains(key)
}

public fun mutate_table_entry(foo: &mut Foo, key: address) {
    let table = field::borrow_mut<String, Table<address, u64>>(
        &mut foo.id,
        ascii::string(b"table")
    );
    let val = table.borrow_mut(key);
    *val = *val + 1;
}

public fun mutate_two_entry_values(foo: &mut Foo, key: address, key_2: address) {
    let table = field::borrow_mut<String, Table<address, u64>>(
        &mut foo.id,
        ascii::string(b"table")
    );
    let val = table.borrow_mut(key);
    *val = *val + 1;

    let val_2 = table.borrow_mut(key_2);
    *val_2 = *val_2 + 1;
}

public fun remove_entry(foo: &mut Foo, key: address): u64 {
    let table = field::borrow_mut<String, Table<address, u64>>(
        &mut foo.id,
        ascii::string(b"table")
    );
    table.remove(key)
}
