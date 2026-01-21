/**
 * This file was automatically generated and represents a Move program.
 * For more information, please see [The Move to Stylus compiler](https://github.com/rather-labs/move-stylus-poc).
 */

// SPDX-License-Identifier: MIT-OR-APACHE-2.0
pragma solidity ^0.8.23;

interface Generics1 {

    struct GenericStructUint128Uint256 {
        uint128 a;
        uint256 b;
    }

    struct GenericStructUint32Uint64 {
        uint32 a;
        uint64 b;
    }

    function testGenericStructs(uint32 a1, uint64 a2, uint128 b1, uint256 b2) external returns (GenericStructUint32Uint64, GenericStructUint128Uint256);

}
