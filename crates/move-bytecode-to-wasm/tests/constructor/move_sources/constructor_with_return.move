module test::constructor_with_return;

use stylus::{
    tx_context::TxContext, 
    object::{Self, UID}
};

public struct Foo has key {
    id: UID,
    value: u64
}

// An init function with returns is not a proper constructor.
// Sui move allows this but we don't.
entry fun init(ctx: &mut TxContext): Foo {
  let foo = Foo {
    id: object::new(ctx),
    value: 101,
  };

  foo
}
