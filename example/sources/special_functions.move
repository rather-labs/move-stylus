module hello_world::special_functions;

use stylus::tx_context::TxContext;
use stylus::object as object;
use stylus::object::UID;
use stylus::transfer as transfer;
use stylus::types as types;
use stylus::event::emit;

public struct Foo has key {
    id: UID,
    value: u64
}

#[ext(event, indexes = 1)]
public struct FooEvent has copy, drop {}

public struct SPECIAL_FUNCTIONS has drop {}

fun init(otw: SPECIAL_FUNCTIONS, ctx: &mut TxContext) {

  assert!(types::is_one_time_witness(&otw), 0);

  let foo = Foo {
    id: object::new(ctx),
    value: 101,
  };

  transfer::share_object(foo);
}

#[ext(payable)]
entry fun receive() {
    emit(FooEvent { });
}

#[ext(payable)]
entry fun fallback(foo: &mut Foo) {
    foo.value = foo.value + 1;
}