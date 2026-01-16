module test::receive_bad_args_1;

use stylus::tx_context::TxContext;

#[ext(abi(payable))]
entry fun receive(_arg1: u64, _arg2: TxContext) {
  // Do nothing
}
