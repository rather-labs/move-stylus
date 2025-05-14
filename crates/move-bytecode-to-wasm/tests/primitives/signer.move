module 0x01::signer_type;

public fun echo(x: signer): signer {
    x
}

public fun echo_identity(x: signer): signer {
  identity(x)
}

fun identity(x: signer): signer{
  x
}

public fun echo_with_int(x: signer, y: u8): (u8, signer) {
    (y, x)
}
