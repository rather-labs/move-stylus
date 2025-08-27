module test::constructor_bad_args_5;

use stylus::tx_context::TxContext;
use stylus::object as object;
use stylus::object::UID;
use stylus::transfer as transfer;
use stylus::types as types;

public struct Foo has key {
    id: UID,
    value: u64
}

// OTW
public struct CONSTRUCTOR_BAD_ARGS_5 has drop {}

// An init function can only take an OTW as first argument and a TxContext as last argument,
// To be considered a constructor.
fun init(ctx: &mut TxContext, otw: CONSTRUCTOR_BAD_ARGS_5) {

  let foo = Foo {
    id: object::new(ctx),
    value: 101,
  };

  transfer::share_object(foo);
}