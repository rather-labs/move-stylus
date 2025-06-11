// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract Echo {
    struct Foo {
        uint64 x;
        bool y;
    }

    function echo(Foo memory foo) public pure returns (Foo memory) {
        return foo;
    }
}
