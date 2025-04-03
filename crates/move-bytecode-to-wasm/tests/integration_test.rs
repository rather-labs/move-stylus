use std::path::Path;
use std::process::Command;

use move_package::BuildConfig;
use move_bytecode_to_wasm::translate_package;
use wasmtime::{Engine, Instance, Module, Store};

mod test {
    use super::*;

    #[test]
    fn test_integration() {

    }

}