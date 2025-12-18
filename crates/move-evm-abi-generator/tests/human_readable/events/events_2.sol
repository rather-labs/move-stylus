/**
 * This file was automatically generated and represents a Move program.
 * For more information, please see [The Move to Stylus compiler](https://github.com/rather-labs/move-stylus-poc).
 */

// SPDX-License-Identifier: MIT-OR-APACHE-2.0
pragma solidity ^0.8.23;

interface Events2 {

    event TestEvent1(uint32 indexed a, address indexed b, uint128 c, uint8[] d, NestedStruct e);
    event TestEvent2(uint32 indexed a, address indexed b, NestedStruct indexed c);
    event TestEvent3(uint32 indexed a, uint8[] indexed b, NestedStruct[] indexed c);
    event TestEvent4(uint64 indexed a, string b);
    event TestEvent5(uint64 indexed a, string indexed b);

    struct NestedStruct {
        uint32 a;
        address b;
        uint128 c;
    }

    function emitTestEvent1(uint32 a, address b, uint128 c, uint8[] d, NestedStruct e) external;
    function emitTestEvent2(uint32 a, address b, NestedStruct c) external;
    function emitTestEvent3(uint32 a, uint8[] b, NestedStruct[] c) external;
    function emitTestEvent4(uint64 a, string b) external;
    function emitTestEvent5(uint64 a, string b) external;

}
