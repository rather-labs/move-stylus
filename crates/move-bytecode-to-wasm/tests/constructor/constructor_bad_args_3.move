module test::constructor_bad_args_3;

use stylus::tx_context::TxContext;
use stylus::object as object;
use stylus::object::UID;
use stylus::transfer as transfer;

public struct Foo has key {
    id: UID,
    value: u64
}

// This One Time Witness is not a valid because it does not match the module name.
public struct CONSTRUCTOR_BAD_ARGS has drop {}

// An init function can only take an OTW as first argument and a TxContext as last argument,
// To be considered a constructor.
fun init(otw: CONSTRUCTOR_BAD_ARGS, ctx: &mut TxContext) {
  let foo = Foo {
    id: object::new(ctx),
    value: 101,
  };

  transfer::share_object(foo);
}