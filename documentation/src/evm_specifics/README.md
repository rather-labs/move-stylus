# EVM Specifics

This compiler is based on the SUI's version of the Move language, which is tailored for the SUI blockchain. Since Stylus targets the EVM, some adjustements were made to ensure compatibility with EVM's architecture and execution model.

Some of those adjustments are coded directly inside the compiler (like the ABI encoding/decoding or the adapted [object model](../object_model/README.md), while others are implemented as part of the [Stylus Framework](../stylus_framework/README.md), a library that provides EVM-compatible abstractions and utilities for Move developers.

In this chapter we are going to cover all the EVM-specific features supported.
