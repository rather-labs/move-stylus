# Stylus Framework

Stylus Framework provides EVM/Stylus specific feature such as cross-contract calls, events, storage functions, as well as high-level abstractions like the [Peep API](./peep.md) and [Account module](./account.md) to facilitate smart contract development on the Stylus platform.

## Exported address

The Stylus Framework is exported at address `0x2`. It can also be used via the alias `stylus`.

## Content

The Stylus Framework includes the following modules:

| Module                            | Description                                                                  | Section                                                                      |
| --------------------------------- | ---------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| `stylus::account`                 | Provides functionalities for managing Stylus accounts                        | -                                                                            |
| `stylus::contract_calls`          | Implements functionalities to perform cross-contract calls.                  | [Cross contract calls](../evm_specifics/cross_contract_calls.md)             |
| `stylus::dynamic_fields`          | Implements dynamic fields for flexible data storage.                         | [Dynamic Fields](./dynamic_fields.md)                                        |
| `stylus::dynamic_fields_named_id` | Implements dynamic fields for flexible data storage for NamedIds.            | [Dynamic Fields](../advanced_programmability/dynamic_fields.md)              |
| `stylus::error`                   | Provides error handling functionalities specific to EVM.                     | [Errors](../evm_specifics/errors.md)                                         |
| `stylus::events`                  | Provides functionalities for emitting.                                       | [Events](../evm_specifics/events.md)                                         |
| `stylus::object`                  | Provides utilities for working with Stylus objects model.                    | [Object Model](../object_model/README.md)                                    |
| `stylus::peep`                    | Allows to read objects owned by other accountes.                             | [Peep API](./peep_api.md)                                                    |
| `stylus::sol_types`               | Contains types that map to Solidity types.                                   | [Solidity Types](./solidity_types.md)                                        |
| `stylus::table`                   | Provides a table data structure for key-value storage.                       | [Dynamic Fields](./dynamic_fields.md#Tables)                                 |
| `stylus::tx_context`              | Provides access to transaction context information.                          | [Transaction Context](../evm_specifics/transaction_context.md)               |

<!--

TODO

Implicit Imports
Just like with Standard Library, some of the modules and types are imported implicitly in the Sui Framework. This is the list of modules and types that are available without explicit use import:

sui::object
sui::object::ID
sui::object::UID
sui::tx_context
sui::tx_context::TxContext
sui::transfer
Source Code
The source code of the Sui Framework is available in the Sui repository.
-->
