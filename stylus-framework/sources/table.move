module stylus::table;

use stylus::object::UID;
use stylus::object as object;
use stylus::tx_context::TxContext;

public struct Table<phantom K: copy + drop + store, phantom V: store> has key, store {
    /// the ID of this table
    id: UID,
    /// the number of key-value pairs in the table
    size: u64,
}

/// Creates a new, empty table
public fun new<K: copy + drop + store, V: store>(ctx: &mut TxContext): Table<K, V> {
    Table {
        id: object::new(ctx),
        size: 0,
    }
}

/// Returns true if there is a value associated with the key `k: K` in table `table: &Table<K, V>`
public native fun contains<K: copy + drop + store, V: store>(table: &Table<K, V>, k: K): bool;

/// Immutable borrows the value associated with the key in the table `table: &Table<K, V>`.
/// Aborts with `sui::dynamic_field::EFieldDoesNotExist` if the table does not have an entry with
/// that key `k: K`.
public native fun borrow<K: copy + drop + store, V: store>(table: &Table<K, V>, k: K): &V;

public native fun borrow_mut<K: copy + drop + store, V: store>(table: &Table<K, V>, k: K): &mut V;

public native fun add<K: copy + drop + store, V: store>(table: &mut Table<K, V>, k: K, v: V);
