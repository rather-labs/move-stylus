module stylus::fallback;

/// Represents the calldata passed to the fallback function.
public struct Calldata has drop, copy {}

/// Returns the calldata as a vector<u8>.
/// In Solidity, fallback function calldata is represented as a byte array.
/// Move does not support this natively, so vector<u8> is the closest equivalent.
public native fun calldata_as_vector(self: &Calldata): vector<u8>;

/// Returns the length of the raw calldata.
public native fun calldata_length(self: &Calldata): u32;

public use fun calldata_as_vector as Calldata.as_vec;
public use fun calldata_length as Calldata.len;