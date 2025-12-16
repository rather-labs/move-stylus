module stylus::bytes;

public use fun as_vec_bytes4 as Bytes4.as_vec;

public struct Bytes4 has copy, drop {}

public native fun as_vec_bytes4(value: &Bytes4): vector<u8>;