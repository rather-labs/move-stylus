module test::receive_bad_returns;

#[ext(payable)]
entry fun receive(): u64 {
  // Do nothing
  101
}
