/**
 * This file was automatically generated and represents a Move program.
 * For more information, please see [The Move to Stylus compiler](https://github.com/rather-labs/move-stylus-poc).
 */

// SPDX-License-Identifier: MIT-OR-APACHE-2.0
pragma solidity ^0.8.23;

interface MiscStructs {

    struct GenericStructUint32Uint128 {
        uint32 a;
        uint128 b;
        GenericEnum c;
        SimpleEnum d;
        NestedStructUint32Uint128 e;
    }

    struct NestedStructUint32Uint128 {
        bool a;
        bool[] b;
    }

    enum GenericEnum {
        A,
        B,
    }

    enum SimpleEnum {
        A,
        B,
    }

    function testMisc(uint32 a, uint128 b, GenericEnum c, SimpleEnum d, NestedStructUint32Uint128 e) external returns (GenericStructUint32Uint128);

}
