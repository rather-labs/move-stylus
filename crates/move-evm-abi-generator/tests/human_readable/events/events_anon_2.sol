/**
 * This file was automatically generated and represents a Move program.
 * For more information, please see [The Move to Stylus compiler](https://github.com/rather-labs/move-stylus).
 */

// SPDX-License-Identifier: MIT-OR-APACHE-2.0
pragma solidity ^0.8.23;

interface EventsAnon2 {

    event Anonymous(NestedStruct indexed a, NestedStructWithEnum indexed b) anonymous;
    event Anonymous2(EventEnum indexed a, EventEnum[] indexed b, NestedStructWithEnum[] indexed c) anonymous;
    event TestEvent1Anon(uint32 indexed a, address indexed b, NestedStruct indexed c) anonymous;
    event TestEvent2Anon(uint32 indexed a, uint8[] indexed b, NestedStruct indexed c) anonymous;
    event TestEvent3Anon(uint64 indexed a, string b) anonymous;
    event TestEvent4Anon(uint64 indexed a, string indexed b) anonymous;

    struct NestedStruct {
        uint32 a;
        address b;
        uint128 c;
    }

    struct NestedStructWithEnum {
        EventEnum a;
        EventEnum[] b;
    }

    enum EventEnum {
        EVENT_1,
        EVENT_2,
        EVENT_3,
    }

    function emitTestAnonEvent1(uint32 a, address b, uint32 c, address d, uint128 e) external;
    function emitTestAnonEvent2(uint32 a, uint8[] b, uint32 c, address d, uint128 e) external;
    function emitTestAnonEvent3(uint64 a, string b) external;
    function emitTestAnonEvent4(uint64 a, string b) external;
    function emitTestAnonymous1(NestedStruct a, NestedStructWithEnum b) external;
    function emitTestAnonymous2(EventEnum p1, EventEnum[] p2, NestedStructWithEnum[] p3) external;

}
