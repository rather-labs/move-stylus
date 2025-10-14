module 0x01::signer_type;

entry fun echo(x: signer, _y: signer): signer {
    x
}
