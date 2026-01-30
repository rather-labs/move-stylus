# Moving stylus

## Overview

This repository contains the source code to compile the Move language to WASM, for running it in [Arbitrum's Stylus Environment](https://docs.arbitrum.io/stylus/gentle-introduction).

The Move-to-Stylus compiler translates Move bytecode directly into WASM. This approach:
- Leverages the existing Move compiler for type checks and validations.
- Avoids modifying the Move compiler.
- Focuses on generating Stylus-compatible WASM modules.

## Disclaimer

> [!WARNING]
> The code has not been audited and may contain security vulnerabilities or bugs. Use at your own risk.

## Move compiler
The Move compiler, based on the [Sui Move implementation](https://github.com/MystenLabs/sui/tree/main/external-crates/move/crates/move-compiler), passes and validate the move code through several stages and return Move-bytecode.

Move bytecode is a high-level intermediate representation interpreted by the Move Virtual Machine (MoveVM). For example:

```rust
const ITEM_PRICE: u64 = 100;

public fun hello_world(): u64 {
    ITEM_PRICE
}
```

```rust
// Move bytecode v6
module 0.hello_world {
    public hello_world(): u64 {
        B0:
            0: LdConst[0](u64: 100)
            1: Ret
    }
    Constants [
        0 => u64: 100
    ]
}
```


## Translation Approach
### Why Direct Translation?
- **Advantages**:
  - Simplifies the setup by avoiding external backends like LLVM.
  - Embeds Stylus-specific interfaces directly in the WASM output.
  - Keeps the entire toolchain in Rust.
- **Trade-offs**:
  - Lacks optimization passes (relies on the Move compiler).
  - Requires implementing features like type operations and testing frameworks from scratch.

---

## WASM Compilation

Stylus programs require a specific structure and interface in WASM. The `user_entrypoint` function serves as the main entry point, handling all function calls. It uses Stylus host functions to read arguments and write results:

```rust
pub extern "vm_hooks" fn read_args(dest: *u8) void;
pub extern "vm_hooks" fn write_result(data: *const u8, len: usize) void;
```

A basic entry point example (pseudocode):

```rust
fn user_entrypoint(len: i32) -> i32 {
    let buffer = read_args(len);
    unpack_abi(buffer); // Load data into the stack
    let return_value = transfer(); // Example function call
    write_result(return_value, len(return_value));
    0 // Success
}
```


## Stylus Router for Move Contracts

To expose all public functions in a Move contract, a single `user_entrypoint` function acts as a router. It uses the function selector (first 4 bytes of calldata) to route calls to the appropriate function (pseudocode):

```rust
fn user_entrypoint(len: i32) -> i32 {
    let buffer = vec![len];
    read_args(&mut buffer);

    let selector = &buffer[0..4];
    match selector {
        function_x_selector => {
            unpack_abi(buffer);
            let return_value = call_function_x();
            write_result(return_value, len(return_value));
        },
        function_y_selector => {
            unpack_abi(buffer);
            let return_value = call_function_y();
            write_result(return_value, len(return_value));
        },
        _ => panic!("Unknown function selector"),
    }

    0 // Success
}
```

## Tooling

The entire compiler is built in Rust, using the following libraries:
- **Move Compiler**: [Sui Move](https://github.com/MystenLabs/sui/tree/main/external-crates/move).
- **WASM Generation**: [Walrus](https://github.com/rustwasm/walrus) for building WASM modules.
- **WASM Debugging**: [wasmprinter](https://github.com/bytecodealliance/wasmprinter) and [wasmparser](https://github.com/bytecodealliance/wasmparser).
- **WASM Testing**: [Wasmtime](https://wasmtime.dev) for creating a WASM runtime in Rust.

## Build instructions
Set up the stylus environment and install required tools:
```bash
make setup-stylus
make install-wasm-tools
```

Or install the `move-stylus` CLI tool:
```bash
make install
```


## Examples

In the `examples/` folder, several fully functional contracts showcase different aspects of the Move language semantics, including core examples, token standards, advanced patterns, and testing & demonstration contracts.

build the example package:
```bash
make build-example
```


deploy to arbitrum dev node (local):
```bash
make deploy-example
make deploy-example-2
make deploy-example-primitives
make deploy-counter
make deploy-counter-with-init
```

Check [Makefile](./Makefile) for more contracts.

run test interactions (make sure to setup a `.env` file):
```bash
make example-interaction
make example-interaction-2
make example-interaction-primitives
make example-counter
make example-counter-with-init
make example-dog-walker
```

Check [Makefile](./Makefile) for more contracts.

## License
This project is developed by [Rather Labs](https://ratherlabs.com) and licensed under the Business Source License 1.1
