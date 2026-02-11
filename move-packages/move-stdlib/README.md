# Move Standard Library

The **Move Standard Library** (`MoveStdlib`) provides fundamental data structures, types, and utilities for Move development. This package is located at address `0x1` and serves as the foundation for all Move contracts.

## Overview

The Move Standard Library is a comprehensive collection of core modules that provide essential functionality for Move smart contract development. It includes primitive type utilities, data structures, string handling, and mathematical operations.

## Key Modules

### Primitive Types
- **u8, u16, u32, u64, u128, u256**: Unsigned integer utilities and operations
- **address**: Address type utilities and manipulations

### Fixed Point Arithmetic
- **fixed_point32**: 32-bit fixed-point decimal arithmetic
- **uq32_32**: Unsigned Q32.32 fixed-point numbers
- **uq64_64**: Unsigned Q64.64 fixed-point numbers

### Data Structures
- **vector**: Dynamic arrays with growable capacity and 0-based indexing
- **option**: Optional value type for representing presence/absence of data
- **bit_vector**: Efficient bit-level operations and storage

### String Handling
- **string**: UTF-8 encoded string type and operations
- **ascii**: ASCII string type with validation

### Utilities
- **type_name**: Runtime type information and reflection
- **macros**: Macro definitions for common patterns
- **unit_test**: Testing framework and utilities

