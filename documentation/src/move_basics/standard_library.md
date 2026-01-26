# Standard Library


The Move Standard Library offers functionality for native types and operations. It is a core collection of modules that do not interact with storage but instead provide essential tools for working with and manipulating data. <!-- The Standard Library is the sole dependency of the Sui Framework and is imported alongside it. -->

## Exported address

The Standard Library is exported at address `0x1`. It can also be used via the alias `std`.

## Content

The Stylus Framework includes the following modules:

| Module| Description| Chapter|
| -| -| -|
| `std::string`| Provides basic string operations| [String](./string) |
| `std::ascii`| Provides basic ASCII operations | [String](./string) |
| `std::option`| Implements `Option<T>`| [Option](./option)|
| `std::vector` | Native operations on the vector type| [Vector](./vector)|
| `std::bit_vector`| Provides operations on bit vectors | - |
| `std::fixed_point32` | Provides the `FixedPoint32` type| -                                    |
<!-- | `std::type_name`| Allows runtime _type reflection_| [Type Reflection](./type-reflection) | -->

### Integers

The Move Standard Library provides a set of functions associated with integer types. These functions are split into multiple modules, each associated with a specific integer type. The modules should not be imported directly, as their functions are available on every integer value.

> [!NOTE]
> All of the modules provide the same set of functions. Namely, `max`, `diff`, `divide_and_round_up`, `sqrt` and `pow`.


| Module                                                         | Description                   |
| -------------------------------------------------------------- | ----------------------------- |
| `std::u8`     | Functions for the `u8` type   |
| `std::u16`   | Functions for the `u16` type  |
| `std::u32`   | Functions for the `u32` type  |
| `std::u64`   | Functions for the `u64` type  |
| `std::u128` | Functions for the `u128` type |
| `std::u256` | Functions for the `u256` type |


## Source Code

The source code of the Move Standard Library is available in the
[Move for Stylus repository](https://github.com/rather-labs/move-stylus-dependencies/tree/master/move-stdlib).


