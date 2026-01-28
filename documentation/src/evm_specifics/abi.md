# Abi

The ABI (Application Binary Interface) defined for Move modules compiled for Stylus follow the same specification as the one used in Solidity. This allows for seamless interaction between Move contracts and other EVM-compatible contracts and tools. It also means that existing tools that understand Solidity's ABI can be used to interact with Move contracts compiled for Stylus.

## Function modifiers

You can specify [Solidity's function modifiers](https://docs.soliditylang.org/en/latest/contracts.html#function-modifiers) for Move functions using the `#[ext(abi(...)]` attribute. This attribute allows you to annotate a Move function with one or more modifiers. These modifiers will be used when generating the ABI for the function.

For example:
```move
#[ext(abi(view))]
entry fun read(counter: &Counter): u64 {
    counter.value
}
```

In this example, the `read` function is annotated with the `view` modifier, indicating that it does not modify the state of the contract. This information will be included in the generated ABI, allowing external tools to understand the function's behavior.

Supported modifiers include:

- `view`
- `pure`
- `payable`

## Exporting the ABI

To export the ABI of a module, you can use the `move-stylus` CLI tool. You can run the following command to generate the ABI file:

```bash
move-stylus export-abi
```

Both human readable and JSON format are supported. By default, the ABI will be exported in JSON format. To export the ABI in human readable format, you can use the `-r` flag:

```bash
move-stylus export-abi -r
```

For the example counter in the [Deploy and Interact](./deploy_and_interact.md) section, the exported ABI in human readable format would look like this:

```solidity
/**
 * This file was automatically generated and represents a Move program.
 * For more information, please see [The Move to Stylus compiler](https://github.com/rather-labs/move-stylus).
 */

// SPDX-License-Identifier: MIT-OR-APACHE-2.0
pragma solidity ^0.8.23;

interface Counter {

    event NewUID(bytes32 indexed uid);

    function create() external;
    function increment(bytes32 counter) external;
    function read(bytes32 counter) view external returns (uint64);
    function setValue(bytes32 counter, uint64 value) external;

}
```

