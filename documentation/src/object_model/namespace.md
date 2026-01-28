# Objects Namespace

To maintain compatibility with the EVM, Move objects are organized within a global storage layout. Conceptually, the storage can be represented by the following Solidity mapping:

```Solidity
// Conceptually: owner => object_id => object_data
mapping(bytes32 => mapping(bytes32 => Object<T>)) public Objects;
```
This nested approach serves two primary purposes:

* **Ownership Partitioning**: The outer mapping is keyed by the _owner identifier_. This can be an **account address** or an object `UID` in the case of [wrapped objects](./wrapped_objects.md).

* **Object Retrieval**: The inner mapping is keyed by the unique Object UID, ensuring that data for specific assets can be retrieved effortlessly. 

>[!Note]
The mapping itself is stored at a specific slot, the slot 0.

## Handling different ownership types

The framework distinguishes between different object lifecycles by routing them to specific owner keys within the mapping. This design allows the runtime to handle `owned`, `shared`, and `frozen` objects under a single consistent logic:

| Object State | Owner Key | Purpose |
| :--- | :--- | :--- |
| **Owned** | `address` / `UID` | Objects belonging to a specific account or parent object. |
| **Shared** | `0x1` | Objects made globally accessible for any user to interact with. |
| **Frozen** | `0x2` | Objects made permanently read-only and immutable. |

By utilizing `0x1` and `0x2` as fixed keys, the framework ensures these states are globally unique and easily handled.

## Type Safety and Validation

To enforce strict type safety across the network, the framework prevents "type-casting" at the storage level. Every object's data blob begins with a **Type Hash** header:

* **Offset 0-8**: Stores a 64-bit Type hash derived from the Move struct definition.

* **Offset 8**: Contains the actual serialized fields of the object.

When a contract attempts to [peep](../stylus_framework/peep.md) or load an object, the runtime first verifies that the hash in storage matches the hash of the Move type specified in the code. If they don't match, the transaction reverts.