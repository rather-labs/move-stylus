# Move Packages

This directory contains core Move packages for the Stylus framework, providing essential libraries and utilities for building smart contracts on Arbitrum Stylus.

## Packages

### [move-stdlib](./move-stdlib)

The **Move Standard Library** provides fundamental data structures, types, and utilities for Move development. Located at address `0x1`.

**Key Features:**
- Primitive type utilities (u8, u16, u32, u64, u128, u256, address)
- Data structures (vector, option, bit_vector)
- String handling (UTF-8 and ASCII)
- Fixed-point arithmetic
- Type reflection and testing utilities

### [stylus-framework](./stylus-framework)

The **Stylus Framework** provides specialized libraries for building Stylus smart contracts with EVM compatibility. Located at address `0x2`.

**Key Features:**
- Account management and balance queries
- Cross-contract call infrastructure
- Transaction context access
- Event emission and logging
- Advanced storage patterns (tables, dynamic fields)
- EVM type conversions
- Testing framework

