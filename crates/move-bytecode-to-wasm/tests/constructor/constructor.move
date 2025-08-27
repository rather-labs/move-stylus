module test::constructor;

use stylus::tx_context::TxContext;
use stylus::object as object;
use stylus::object::UID;
use stylus::transfer as transfer;

public struct Foo has key {
    id: UID,
    value: u64
}

fun init(ctx: &mut TxContext) {
  let foo = Foo {
    id: object::new(ctx),
    value: 101,
  };

  transfer::share_object(foo);
}

public fun read_value(foo: &Foo): u64 {
    foo.value
}

public fun set_value(foo: &mut Foo, value: u64) {
    foo.value = value;
}
