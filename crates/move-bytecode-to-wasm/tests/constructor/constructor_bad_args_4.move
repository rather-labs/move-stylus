module test::constructor_bad_args_4;

public struct CONSTRUCTOR_BAD_ARGS_4 has drop {}

// An init function can only take an OTW as first argument and a TxContext as last argument,
// To be considered a constructor.
fun init(otw: CONSTRUCTOR_BAD_ARGS_4) {}