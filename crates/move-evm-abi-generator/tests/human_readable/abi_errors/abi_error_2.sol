/**
 * This file was automatically generated and represents a Move program.
 * For more information, please see [The Move to Stylus compiler](https://github.com/rather-labs/move-stylus-poc).
 */

// SPDX-License-Identifier: MIT-OR-APACHE-2.0
pragma solidity ^0.8.23;

interface AbiError2 {

    error CustomError(string error_message, uint64 error_code);
    error CustomError2(bool a, uint8 b, uint16 c, uint32 d, uint64 e, uint128 f, uint256 g, address h);

    function revertCustomError(string s, uint64 code) external;
    function revertCustomError2(bool a, uint8 b, uint16 c, uint32 d, uint64 e, uint128 f, uint256 g, address h) external;

}
