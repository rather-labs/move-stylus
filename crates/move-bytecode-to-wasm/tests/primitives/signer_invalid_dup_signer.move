module 0x01::signer_type;

public fun echo(x: signer, _y: signer): signer {
    x
}
