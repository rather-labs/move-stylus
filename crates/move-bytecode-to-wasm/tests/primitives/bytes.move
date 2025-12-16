module 0x01::bytes;
use stylus::bytes::Bytes4;

entry fun test_bytes4_as_vec(b: &Bytes4): vector<u8> {
    b.as_vec()
}