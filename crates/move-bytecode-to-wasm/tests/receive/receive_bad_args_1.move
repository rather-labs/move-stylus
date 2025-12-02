module test::receive_bad_args_1;

use stylus::tx_context::TxContext;

#[ext(payable)]
entry fun receive(arg1: u64, arg2: TxContext) {
  // Do nothing
}
