module 0x01::ref_signer;


public fun dummy(_s: &signer) {
    // Does nothing, but forces a borrow
}

public fun use_dummy(s: signer): signer {
    dummy(&s); 
    s
}
