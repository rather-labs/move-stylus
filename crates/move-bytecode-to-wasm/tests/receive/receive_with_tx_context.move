module test::receive_with_tx_context;

use stylus::tx_context::TxContext;
#[ext(payable)]
entry fun receive(ctx: &TxContext) {
  // Do nothing
}
