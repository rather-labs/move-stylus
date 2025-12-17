module 0x01::bytes;
use stylus::bytes::{Bytes1, Bytes2, Bytes4, Bytes8, Bytes16, Bytes32};

entry fun test_bytes4_as_vec(b: &Bytes4): vector<u8> {
    b.as_vec()
}

entry fun test_bytes1_as_vec(b: &Bytes1): vector<u8> {
    b.as_vec()
}

entry fun test_bytes2_as_vec(b: &Bytes2): vector<u8> {
    b.as_vec()
}

entry fun test_bytes8_as_vec(b: &Bytes8): vector<u8> {
    b.as_vec()
}

entry fun test_bytes16_as_vec(b: &Bytes16): vector<u8> {
    b.as_vec()
}

entry fun test_bytes32_as_vec(b: &Bytes32): vector<u8> {
    b.as_vec()
}

entry fun test_mixed_bytes_as_vec(a: &Bytes4, b: &Bytes4, c: &Bytes8, d: &Bytes16): bool {
    let mut vec = a.as_vec();
    vector::append(&mut vec, b.as_vec());
    vector::append(&mut vec, c.as_vec());
   
    vec == d.as_vec()
}