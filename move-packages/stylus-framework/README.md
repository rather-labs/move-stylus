# Stylus Framework

The **Stylus Framework** (`StylusFramework`) provides specialized libraries for building Stylus smart contracts that can interact with the Ethereum Virtual Machine (EVM). This package is located at address `0x2` and depends on `move-stdlib`.

## Overview

The Stylus Framework bridges Move smart contracts with Arbitrum Stylus, enabling seamless interaction with the Ethereum ecosystem. It provides EVM-compatible modules for account management, cross-contract calls, event emission, and advanced storage patterns.

## Key Modules

- **`account.move`**
  Provides functions for querying account information on the blockchain.
- **`contract_calls.move`**
  Enables cross-contract calls with support for gas limits, value transfers, and delegate calls.
- **`dynamic_field.move`**
  Implements dynamic fields that can be added to objects after construction. Provides functions for adding, borrowing, removing, and checking existence of dynamic fields.
- **`dynamic_field_named_id.move`**
  Extends dynamic field functionality to work with `NamedId` objects, providing a wrapper around `dynamic_field` operations for objects with named identifiers.
- **`error.move`**
  Provides the `revert` function for reverting the current transaction with a custom error value.
- **`event.move`**
  Provides functions for emitting ABI-encoded [events/logs](https://docs.arbitrum.io/stylus-by-example/basic_examples/events).
- **`object.move`**
  Manages storage-backed objects.
- **`sol_types.move`**
  Defines Solidity-compatible fixed-size byte types (`Bytes1` through `Bytes32`) and provides conversion functions to transform these types into `vector<u8>` for interoperability with Solidity contracts.
- **`table.move`**
  Implements a map-like collection where keys and values are stored using the object system rather than within the `Table` value itself. The `Table` struct acts as a handle for retrieving entries. Provides standard map operations: add, borrow, remove, and query operations like `contains`, `length`, and `is_empty`.
- **`transfer.move`**
  Implements object transfer functions that enforce Sui's ownership model:
  - `transfer`: moves an object to a single owner; only the owner can read and write it.
  - `share_object`: shares an object; once shared, it can be read and written by anyone.
  - `freeze_object`: freezes an object; once frozen, it can be read by anyone but not modified.
- **`tx_context.move`**
  Defines the `TxContext` object, which provides methods for accessing information about the current transaction.

## Features

- **EVM Compatibility**: Direct integration with Ethereum opcodes and host functions
- **Cross-Contract Calls**: Safe and configurable contract-to-contract communication
- **Advanced Storage**: Dynamic fields and table-based storage patterns
- **Event System**: Compatible event emission for EVM event logs
- **Testing Framework**: Comprehensive testing utilities for Stylus contracts

