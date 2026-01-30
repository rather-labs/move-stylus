/**
 * This file was automatically generated and represents a Move program.
 * For more information, please see [The Move to Stylus compiler](https://github.com/rather-labs/move-stylus).
 */

// SPDX-License-Identifier: MIT-OR-APACHE-2.0
pragma solidity ^0.8.23;

interface Events1 {

    event TestEvent1(uint32 indexed n);
    event TestEvent2(uint32 indexed a, address indexed b, uint128 indexed c);
    event TestEvent3(uint32 indexed a, address indexed b, uint128 c, uint8[] d);
    event TestEvent4(uint32 indexed a, address indexed b, uint8[] indexed c);

    function emitTestEvent1(uint32 n) external;
    function emitTestEvent2(uint32 a, address b, uint128 c) external;
    function emitTestEvent3(uint32 a, address b, uint128 c, uint8[] d) external;
    function emitTestEvent4(uint32 a, address b, uint8[] c) external;

}
