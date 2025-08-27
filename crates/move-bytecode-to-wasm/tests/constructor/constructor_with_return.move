module test::constructor_with_return;

use stylus::tx_context::TxContext;
use stylus::object as object;
use stylus::object::UID;
use stylus::transfer as transfer;

public struct Foo has key {
    id: UID,
    value: u64
}

// An init function with returns is not a proper constructor.
// Sui move allows this but we don't.
fun init(ctx: &mut TxContext): Foo {
  let foo = Foo {
    id: object::new(ctx),
    value: 101,
  };

  foo
}