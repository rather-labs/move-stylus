module test::constructor_bad_args_1;

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
entry fun init(value: u64, _value_2: u64, ctx: &mut TxContext) {
  let foo = Foo {
    id: object::new(ctx),
    value: value,
  };

  transfer::share_object(foo);
}
