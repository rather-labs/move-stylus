module 0x01::signer_type;

// Echoes the signer
public fun echo(s: signer): signer {
  identity(s)
}

public fun echo_2(s1: signer, s2: signer): (signer, signer) {
  identity_2(s1, s2)
}

fun identity(x: signer): signer {
  x
}

fun identity_2(x: signer, y: signer): (signer, signer) {
    (x, y)
}
