module test::constructor_bad_args_1;

use stylus::tx_context::TxContext;
use stylus::object as object;
use stylus::object::UID;
use stylus::transfer as transfer;

public struct Foo has key {
    id: UID,
    value: u64
}

// An init function can only take an OTW as first argument and a TxContext as last argument,
// To be considered a constructor.
fun init(value: u64, value_2: u64, ctx: &mut TxContext) {
  let foo = Foo {
    id: object::new(ctx),
    value: value,
  };

  transfer::share_object(foo);
}