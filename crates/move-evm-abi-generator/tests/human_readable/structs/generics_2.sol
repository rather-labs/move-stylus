/**
 * This file was automatically generated and represents a Move program.
 * For more information, please see [The Move to Stylus compiler](https://github.com/rather-labs/move-stylus-poc).
 */

// SPDX-License-Identifier: MIT-OR-APACHE-2.0
pragma solidity ^0.8.23;

interface Generics2 {

    struct GenericStructUint128Uint256 {
        uint128 a;
        uint256 b;
    }

    struct GenericStructUint32Uint64 {
        uint32 a;
        uint64 b;
    }

    function testGenericStructs(GenericStructUint32Uint64 s1, GenericStructUint128Uint256 s2) external (uint32, uint64, uint128, uint256);
    function unpack1(GenericStructUint32Uint64 s) external (uint32, uint64);
    function unpack2(GenericStructUint128Uint256 s) external (uint128, uint256);

}
