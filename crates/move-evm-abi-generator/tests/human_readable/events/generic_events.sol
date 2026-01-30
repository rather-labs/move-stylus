/**
 * This file was automatically generated and represents a Move program.
 * For more information, please see [The Move to Stylus compiler](https://github.com/rather-labs/move-stylus).
 */

// SPDX-License-Identifier: MIT-OR-APACHE-2.0
pragma solidity ^0.8.23;

interface GenericEvents {

    event TestEvent1(address indexed a, NestedStructBoolUint256 indexed b);
    event TestEvent1(uint16 indexed a, NestedStructUint32Uint64 indexed b);

    struct NestedStructBoolUint256 {
        bool a;
        uint256 b;
        bool[] c;
        uint256[] d;
    }

    struct NestedStructUint32Uint64 {
        uint32 a;
        uint64 b;
        uint32[] c;
        uint64[] d;
    }

    function testEventAddressBoolU256(address a, NestedStructBoolUint256 b) external;
    function testEventU16U32U64(uint16 a, NestedStructUint32Uint64 b) external;

}
