module test::constructor_bad_args_2;

use stylus::tx_context::TxContext;
use stylus::object::{Self};
use stylus::object::UID;
use stylus::transfer::{Self};

public struct Foo has key {
    id: UID,
    value: u64
}

// An init function can only take an OTW as first argument and a TxContext as last argument,
// To be considered a constructor.
entry fun init(ctx: &mut TxContext, value: u64) {
  let foo = Foo {
    id: object::new(ctx),
    value: value,
  };

  transfer::share_object(foo);
}