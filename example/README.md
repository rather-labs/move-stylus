# Examples

In this folder, among contracts that only demonstrates the Move Language capabilities, there are several fully functional contracts that showcase different aspects of the Move language semantics:

### Core Examples

- **`counter.move`**
  - Uses the `share_object` function to make counters globally accessible so anyone can increment their value.
  - Emits an event for each created counter so users can capture its ID.
  - Allows seamless retrieval of objects from storage by their ID.
  - Enforces access control in the `set_value` function using the `TxContext.sender` method.

- **`counter_with_init.move`**
  - Same as `counter.move`, but the counter is created with a constructor function (`init`).

- **`counter_named_id.move`**
  - Variant of the counter contract using `NamedId` for object identification instead of standard `UID`.
  - Demonstrates named object management with deterministic addressing.
  - Provides the same functionality as `counter.move` with alternative ID management.

- **`dog_walker.move`**
  - Enforces access control using the capability pattern.
  - Uses the `transfer` function to assign the capability to a unique owner.
  - Emits an event when the dog goes out for a walk.
  - Prevents the action if the contract is not called by the dog's owner.

### Token Standards

- **`erc20.move`**
  - Full implementation of the ERC-20 token standard.
  - Includes mint, burn, transfer, and approval functionality.
  - Uses dynamic fields for storing balances and allowances.
  - Emits standard Transfer and Approval events.
  - Demonstrates initialization with constructor (`init`) creating shared and frozen objects for token metadata.

- **`erc721.move`**
  - Complete implementation of the ERC-721 (NFT) standard with metadata extension.
  - Supports minting, burning, transferring, and approving NFTs.
  - Implements operator approvals for managing multiple tokens.
  - Uses dynamic fields for tracking ownership, balances, and approvals.
  - Includes safe transfer checks for receiver contracts.
  - Demonstrates interface detection via `supports_interface`.

### Advanced Patterns

- **`delegated_counter.move` & `delegated_counter_logic_*.move`**
  - Demonstrates the proxy/delegate pattern for upgradeable contracts.
  - Main contract delegates increment operations to external logic contracts.
  - Supports changing the logic address dynamically via `change_logic`.
  - Shows how to maintain state while upgrading contract logic.
  - Tests delegation with modifications before and after delegated calls.

- **`delegated_counter_named_id.move` & `delegated_counter_named_id_logic_*.move`**
  - Same delegation pattern as `delegated_counter.move` but using `NamedId` for object identification.
  - Demonstrates how named IDs work with cross-contract calls and delegation.

- **`cross_contract_call.move`**
  - Demonstrates cross-contract calls to external ERC-20 contracts.
  - Shows how to query balance, total supply, and execute transfers on external tokens.
  - Uses the `contract_calls` module from the stylus-framework.

### Testing & Demonstration

- **`hello_world.move`**
  - Comprehensive demonstration of basic Move language features.
  - Includes constants, generic types, structs, enums, and function calls.
  - Tests local variable handling, copying, and moving semantics.

- **`primitives_and_operations.move`**
  - Showcases all supported primitive types and operations.
  - Demonstrates arithmetic, bitwise, boolean, and comparison operations.
  - Includes casting between different integer types.

- **`revert_errors.move`**
  - Demonstrates custom error handling with the `revert` function.
  - Shows ABI-encoded error types with various data structures.
  - Includes examples of standard, custom, and nested struct errors.

- **`simple_counter.move`**
  - Basic counter implementation without storage backing (uses `drop` ability).
  - Includes unit tests demonstrating Move's testing capabilities.
  - Shows test patterns including `#[test]` and `#[expected_failure]` attributes.

- **`stack.move`**
  - Implements a generic stack data structure using the wrapper type pattern.
  - Demonstrates generic programming with type parameters.
  - Shows safe stack operations with `Option` types for empty stack handling.


