module test::constructor_with_otw;

use stylus::tx_context::TxContext;
use stylus::object as object;
use stylus::object::UID;
use stylus::transfer as transfer;
use stylus::types as types;

public struct Foo has key {
    id: UID,
    value: u64
}

public struct CONSTRUCTOR_WITH_OTW has drop {}

// An init function can only take an OTW as first argument and a TxContext as last argument,
// To be considered a constructor.
fun init(otw: CONSTRUCTOR_WITH_OTW, ctx: &mut TxContext) {

  assert!(types::is_one_time_witness(&otw), 0);

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
