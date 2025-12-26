//! Module aggregating all test modules for the smart contract language
//!
//! We put all tests inside a single module to leverage on compiled modules cache. A lot of tests
//! shared compiled code. This way we avoid recompiling the same code multiple times.
pub mod common;
pub mod constructor;
pub mod control_flow;
pub mod dependencies;
pub mod enums;
pub mod framework;
pub mod generic_functions;
pub mod native;
pub mod operations_bitwise;
pub mod operations_cast;
pub mod operations_comparisons;
pub mod operations_equality;
pub mod primitives;
pub mod receive;
pub mod references;
pub mod stdlib;
pub mod storage;
pub mod structs;
