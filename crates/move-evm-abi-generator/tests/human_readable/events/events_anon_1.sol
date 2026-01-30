/**
 * This file was automatically generated and represents a Move program.
 * For more information, please see [The Move to Stylus compiler](https://github.com/rather-labs/move-stylus).
 */

// SPDX-License-Identifier: MIT-OR-APACHE-2.0
pragma solidity ^0.8.23;

interface EventsAnon1 {

    event TestEvent1Anon(uint32 indexed n) anonymous;
    event TestEvent2Anon(uint32 indexed a, address indexed b, uint128 indexed c) anonymous;
    event TestEvent3Anon(uint32 indexed a, address indexed b, uint128 c, uint8[] d) anonymous;
    event TestEvent4Anon(uint32 indexed a, address indexed b, uint128 c, uint8[] d, NestedStruct e) anonymous;
    event TestEvent5Anon(uint32 indexed a, address indexed b, uint8[] indexed c) anonymous;

    struct NestedStruct {
        uint32 a;
        address b;
        uint128 c;
    }

    function emitTestAnonEvent1(uint32 n) external;
    function emitTestAnonEvent2(uint32 a, address b, uint128 c) external;
    function emitTestAnonEvent3(uint32 a, address b, uint128 c, uint8[] d) external;
    function emitTestAnonEvent4(uint32 a, address b, uint128 c, uint8[] d, uint32 e, address f, uint128 g) external;
    function emitTestAnonEvent5(uint32 a, address b, uint8[] c) external;

}
