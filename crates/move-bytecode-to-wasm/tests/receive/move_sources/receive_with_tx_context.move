module test::receive_with_tx_context;

use stylus::tx_context::TxContext;
#[ext(abi(payable))]
entry fun receive(_ctx: &TxContext) {
  // Do nothing
}
