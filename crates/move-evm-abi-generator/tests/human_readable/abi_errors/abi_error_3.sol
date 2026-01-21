/**
 * This file was automatically generated and represents a Move program.
 * For more information, please see [The Move to Stylus compiler](https://github.com/rather-labs/move-stylus-poc).
 */

// SPDX-License-Identifier: MIT-OR-APACHE-2.0
pragma solidity ^0.8.23;

interface AbiError3 {

    error ErrorWithEnum(ErrorEnum a, ErrorEnum[] b);
    error ErrorWithNestedStructs(NestedStruct a, NestedStruct2 b);
    error ErrorWithVectors(uint32[] a, uint128[] b, uint64[][] c);

    struct NestedStruct {
        string pos0;
    }

    struct NestedStruct2 {
        string a;
        uint64 b;
    }

    enum ErrorEnum {
        ERROR_1,
        ERROR_2,
        ERROR_3,
    }

    function revertErrorWithEnum(ErrorEnum a, ErrorEnum[] b) external;
    function revertErrorWithNestedStructs(string a, string b, uint64 c) pure external;
    function revertErrorWithVectors(uint32[] a, uint128[] b, uint64[][] c) pure external;

}
