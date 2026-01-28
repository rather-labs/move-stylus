/// This module implements Solidity-compatible fixed-size byte types for the Stylus framework.
/// It provides Move struct representations for 'bytes1' through 'bytes32'.
module stylus::sol_types;

/// Internal native function to cast a fixed-size byte struct into a raw vector.
/// `n` specifies the expected byte length (1-32).
public(package) native fun as_vec_bytes_n<T: copy + drop>(value: &T, n: u8): vector<u8>;

/// These structs represent the standard Solidity fixed-size byte arrays.
public struct Bytes1 has copy, drop {}
public struct Bytes2 has copy, drop {}
public struct Bytes3 has copy, drop {}
public struct Bytes4 has copy, drop {}
public struct Bytes5 has copy, drop {}
public struct Bytes6 has copy, drop {}
public struct Bytes7 has copy, drop {}
public struct Bytes8 has copy, drop {}
public struct Bytes9 has copy, drop {}
public struct Bytes10 has copy, drop {}
public struct Bytes11 has copy, drop {}
public struct Bytes12 has copy, drop {}
public struct Bytes13 has copy, drop {}
public struct Bytes14 has copy, drop {}
public struct Bytes15 has copy, drop {}
public struct Bytes16 has copy, drop {}
public struct Bytes17 has copy, drop {}
public struct Bytes18 has copy, drop {}
public struct Bytes19 has copy, drop {}
public struct Bytes20 has copy, drop {}
public struct Bytes21 has copy, drop {}
public struct Bytes22 has copy, drop {}
public struct Bytes23 has copy, drop {}
public struct Bytes24 has copy, drop {}
public struct Bytes25 has copy, drop {}
public struct Bytes26 has copy, drop {}
public struct Bytes27 has copy, drop {}
public struct Bytes28 has copy, drop {}
public struct Bytes29 has copy, drop {}
public struct Bytes30 has copy, drop {}
public struct Bytes31 has copy, drop {}
public struct Bytes32 has copy, drop {}

[Image of Solidity fixed-size bytes vs dynamic bytes memory layout]

// --- Conversion Functions ---
// The following functions provide safe wrappers to convert the fixed-size 
// structs into Move 'vector<u8>' types.

public fun as_vec_bytes1(value: &Bytes1): vector<u8> {
    as_vec_bytes_n<Bytes1>(value, 1)
}
public fun as_vec_bytes2(value: &Bytes2): vector<u8> {
    as_vec_bytes_n<Bytes2>(value, 2)
}
public fun as_vec_bytes3(value: &Bytes3): vector<u8> {
    as_vec_bytes_n<Bytes3>(value, 3)
}
public fun as_vec_bytes4(value: &Bytes4): vector<u8> {
    as_vec_bytes_n<Bytes4>(value, 4)
}
public fun as_vec_bytes5(value: &Bytes5): vector<u8> {
    as_vec_bytes_n<Bytes5>(value, 5)
}
public fun as_vec_bytes6(value: &Bytes6): vector<u8> {
    as_vec_bytes_n<Bytes6>(value, 6)
}
public fun as_vec_bytes7(value: &Bytes7): vector<u8> {
    as_vec_bytes_n<Bytes7>(value, 7)
}
public fun as_vec_bytes8(value: &Bytes