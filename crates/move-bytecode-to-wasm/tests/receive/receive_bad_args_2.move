module test::receive_bad_args_2;

#[ext(payable)]
entry fun receive(_arg: u64) {
  // Do nothing
}
