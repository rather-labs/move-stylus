/// Module: todo_list
module hello_world::todo_list;

use std::string::String;
use stylus::tx_context::TxContext;
use stylus::object as object;
use stylus::object::UID;
use stylus::transfer as transfer;

/// List of todos. Can be managed by the owner and shared with others.
public struct TodoList has key, store {
    id: UID,
    items: vector<String>
}

/// Create a new todo list.
public fun new(ctx: &mut TxContext) {
    let todo_list = TodoList {
        id: object::new(ctx),
        items: vector[]
    };

    transfer::share_object(todo_list);
}

/// Add a new todo item to the list.
public fun add(list: &mut TodoList, item: String) {
    list.items.push_back(item);
}

/// Remove a todo item from the list by index.
public fun remove(list: &mut TodoList, index: u64): String {
    list.items.remove(index)
}

/// Delete the list and the capability to manage it.
public fun delete(list: TodoList) {
    object::delete(list);
}

/// Get the number of items in the list.
public fun length(list: &TodoList): u64 {
    list.items.length()
}

/// Read the items in the list.
public fun read(list: &TodoList): vector<String> {
    list.items
}