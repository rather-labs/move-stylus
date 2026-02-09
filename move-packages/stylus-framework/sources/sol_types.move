// Copyright (c) 2025 Rather Labs, Inc.
// SPDX-License-Identifier: BUSL-1.1

/// This module implements Solidity-compatible fixed-size byte types for the Stylus framework.
/// It provides Move struct representations for 'bytes1' through 'bytes32', enabling
/// seamless ABI interoperability with Ethereum smart contracts and precise memory layouts.

module stylus::sol_types;

/// Internal native function to cast a fixed-size byte struct into a raw vector.
/// 'n' specifies the expected byte length (1-32).
public(package) native fun as_vec_bytes_n<T: copy + drop>(value: &T, n: u8): vector<u8>;

// --- Fixed-size Bytes Definitions ---
// These structs represent the standard Solidity fixed-size byte arrays.
// They are primarily used for ABI encoding and matching EVM storage layouts.

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
public fun as_vec_bytes8(value: &Bytes8): vector<u8> {
    as_vec_bytes_n<Bytes8>(value, 8)
}
public fun as_vec_bytes9(value: &Bytes9): vector<u8> {
    as_vec_bytes_n<Bytes9>(value, 9)
}
public fun as_vec_bytes10(value: &Bytes10): vector<u8> {
    as_vec_bytes_n<Bytes10>(value, 10)
}
public fun as_vec_bytes11(value: &Bytes11): vector<u8> {
    as_vec_bytes_n<Bytes11>(value, 11)
}
public fun as_vec_bytes12(value: &Bytes12): vector<u8> {
    as_vec_bytes_n<Bytes12>(value, 12)
}
public fun as_vec_bytes13(value: &Bytes13): vector<u8> {
    as_vec_bytes_n<Bytes13>(value, 13)
}
public fun as_vec_bytes14(value: &Bytes14): vector<u8> {
    as_vec_bytes_n<Bytes14>(value, 14)
}
public fun as_vec_bytes15(value: &Bytes15): vector<u8> {
    as_vec_bytes_n<Bytes15>(value, 15)
}
public fun as_vec_bytes16(value: &Bytes16): vector<u8> {
    as_vec_bytes_n<Bytes16>(value, 16)
}
public fun as_vec_bytes17(value: &Bytes17): vector<u8> {
    as_vec_bytes_n<Bytes17>(value, 17)
}
public fun as_vec_bytes18(value: &Bytes18): vector<u8> {
    as_vec_bytes_n<Bytes18>(value, 18)
}
public fun as_vec_bytes19(value: &Bytes19): vector<u8> {
    as_vec_bytes_n<Bytes19>(value, 19)
}
public fun as_vec_bytes20(value: &Bytes20): vector<u8> {
    as_vec_bytes_n<Bytes20>(value, 20)
}
public fun as_vec_bytes21(value: &Bytes21): vector<u8> {
    as_vec_bytes_n<Bytes21>(value, 21)
}
public fun as_vec_bytes22(value: &Bytes22): vector<u8> {
    as_vec_bytes_n<Bytes22>(value, 22)
}
public fun as_vec_bytes23(value: &Bytes23): vector<u8> {
    as_vec_bytes_n<Bytes23>(value, 23)
}
public fun as_vec_bytes24(value: &Bytes24): vector<u8> {
    as_vec_bytes_n<Bytes24>(value, 24)
}
public fun as_vec_bytes25(value: &Bytes25): vector<u8> {
    as_vec_bytes_n<Bytes25>(value, 25)
}
public fun as_vec_bytes26(value: &Bytes26): vector<u8> {
    as_vec_bytes_n<Bytes26>(value, 26)
}
public fun as_vec_bytes27(value: &Bytes27): vector<u8> {
    as_vec_bytes_n<Bytes27>(value, 27)
}
public fun as_vec_bytes28(value: &Bytes28): vector<u8> {
    as_vec_bytes_n<Bytes28>(value, 28)
}
public fun as_vec_bytes29(value: &Bytes29): vector<u8> {
    as_vec_bytes_n<Bytes29>(value, 29)
}
public fun as_vec_bytes30(value: &Bytes30): vector<u8> {
    as_vec_bytes_n<Bytes30>(value, 30)
}
public fun as_vec_bytes31(value: &Bytes31): vector<u8> {
    as_vec_bytes_n<Bytes31>(value, 31)
}
public fun as_vec_bytes32(value: &Bytes32): vector<u8> {
    as_vec_bytes_n<Bytes32>(value, 32)
}

public use fun as_vec_bytes1 as Bytes1.as_vec;
public use fun as_vec_bytes2 as Bytes2.as_vec;
public use fun as_vec_bytes3 as Bytes3.as_vec;
public use fun as_vec_bytes4 as Bytes4.as_vec;
public use fun as_vec_bytes5 as Bytes5.as_vec;
public use fun as_vec_bytes6 as Bytes6.as_vec;
public use fun as_vec_bytes7 as Bytes7.as_vec;
public use fun as_vec_bytes8 as Bytes8.as_vec;
public use fun as_vec_bytes9 as Bytes9.as_vec;
public use fun as_vec_bytes10 as Bytes10.as_vec;
public use fun as_vec_bytes11 as Bytes11.as_vec;
public use fun as_vec_bytes12 as Bytes12.as_vec;
public use fun as_vec_bytes13 as Bytes13.as_vec;
public use fun as_vec_bytes14 as Bytes14.as_vec;
public use fun as_vec_bytes15 as Bytes15.as_vec;
public use fun as_vec_bytes16 as Bytes16.as_vec;
public use fun as_vec_bytes17 as Bytes17.as_vec;
public use fun as_vec_bytes18 as Bytes18.as_vec;
public use fun as_vec_bytes19 as Bytes19.as_vec;
public use fun as_vec_bytes20 as Bytes20.as_vec;
public use fun as_vec_bytes21 as Bytes21.as_vec;
public use fun as_vec_bytes22 as Bytes22.as_vec;
public use fun as_vec_bytes23 as Bytes23.as_vec;
public use fun as_vec_bytes24 as Bytes24.as_vec;
public use fun as_vec_bytes25 as Bytes25.as_vec;
public use fun as_vec_bytes26 as Bytes26.as_vec;
public use fun as_vec_bytes27 as Bytes27.as_vec;
public use fun as_vec_bytes28 as Bytes28.as_vec;
public use fun as_vec_bytes29 as Bytes29.as_vec;
public use fun as_vec_bytes30 as Bytes30.as_vec;
public use fun as_vec_bytes31 as Bytes31.as_vec;
public use fun as_vec_bytes32 as Bytes32.as_vec;