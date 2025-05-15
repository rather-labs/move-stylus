module 0x01::hello_world;

//  TODO: Add support for native functions
//  native public fun emit_log(ptr: u32, len: u32, topic: u32);

public fun echo(x: u128): u128 {
    x
}


public fun echo_signer_with_int(x: signer, y: u8): (u8, signer) {
    (y, x)
}


public fun ref_u8_arg(y: &u8): u8 {
  *y
}

  // Receives by reference directly
public fun ref_vec_u8_arg(y: &vector<u8>): vector<u8> {
  *y
}

public fun call_ref_u8_internal(x: u8): u8 {
  ref_u8_arg(&x)
}

public fun call_ref_vec_u8_internal(x: vector<u8>): vector<u8> {
  ref_vec_u8_arg(&x)
}