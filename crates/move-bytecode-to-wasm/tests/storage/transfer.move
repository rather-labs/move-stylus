module test::transfer;

use stylus::tx_context::TxContext;
use stylus::object as object;
use stylus::object::UID;
use stylus::transfer as transfer;

public struct Foo has key {
    id: UID,
    value: u64
}

public fun create_shared(ctx: &mut TxContext) {
  let foo = Foo {
    id: object::new(ctx),
    value: 101,
  };

  transfer::share_object(foo);
}

public fun create_owned(recipient: address, ctx: &mut TxContext) {
  let foo = Foo {
    id: object::new(ctx),
    value: 101,
  };

  transfer::transfer(foo, recipient);
}

public fun create_frozen(ctx: &mut TxContext) {
  let foo = Foo {
    id: object::new(ctx),
    value: 101,
  };

  transfer::freeze_object(foo);
}

public fun read_value(foo: &Foo): u64 {
    foo.value
}

public fun set_value(foo: &mut Foo, value: u64) {
    foo.value = value;
}

public fun increment_value(foo: &mut Foo) {
    foo.value = foo.value + 1;
}

// Wrappers to manipulate storage directly: delete, transfer, freeze and share object.
public fun delete_obj(foo: Foo) {
    object::delete(foo);
}

public fun freeze_obj(foo: Foo) { 
  transfer::freeze_object(foo);
}

public fun share_obj(foo: Foo) {
  transfer::share_object(foo);
}

public fun transfer_obj(foo: Foo, recipient: address) { 
  transfer::transfer(foo, recipient);
}

public fun get_foo(foo: &Foo): &Foo {
    foo
}