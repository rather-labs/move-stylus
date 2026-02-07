module test::constructor_bad_args_2;

use stylus::{
    tx_context::TxContext, 
    object::{Self, UID}, 
    transfer::{Self}
};

public struct Foo has key {
    id: UID,
    value: u64
}

// An init function can only take a TxContext as argument
// To be considered a constructor.
entry fun init(ctx: &mut TxContext, value: u64) {
  let foo = Foo {
    id: object::new(ctx),
    value: value,
  };

  transfer::share_object(foo);
}