// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

//! This module contains aux functions used in unit tests in this module
#![allow(dead_code)]
use crate::{
    compilation_context::globals::CompilationContextGlobals, data::DATA_ABORT_MESSAGE_PTR_OFFSET,
    error::RuntimeError, memory::setup_module_memory,
};
use walrus::{FunctionId, MemoryId, Module, ModuleConfig, ValType};
use wasmtime::{Caller, Engine, Instance, Linker, Module as WasmModule, Store, TypedFunc};

// Reserved memory for runtime errors and other data.
// The tests will be write from this point on.
pub const INITIAL_MEMORY_OFFSET: i32 = 2000;

pub fn build_module(
    initial_memory_offset: Option<i32>,
) -> (Module, FunctionId, MemoryId, CompilationContextGlobals) {
    let config = ModuleConfig::new();
    let mut module = Module::with_config(config);

    #[cfg(feature = "inject-host-debug-fns")]
    crate::test_tools::inject_debug_fns(&mut module);

    let (allocator_func, memory_id, compilation_context_globals) = setup_module_memory(
        &mut module,
        Some(initial_memory_offset.unwrap_or(0) + INITIAL_MEMORY_OFFSET),
    );

    (
        module,
        allocator_func,
        memory_id,
        compilation_context_globals,
    )
}

pub fn setup_wasmtime_module<T, U>(
    module: &mut Module,
    initial_memory_data: Vec<u8>,
    function_name: &str,
    linker: Option<Linker<()>>,
) -> (Linker<()>, Instance, Store<()>, TypedFunc<T, U>)
where
    U: wasmtime::WasmResults,
    T: wasmtime::WasmParams,
{
    let linker = if let Some(linker) = linker {
        linker
    } else {
        Linker::new(&Engine::default())
    };

    let engine = linker.engine();

    let module = WasmModule::from_binary(engine, &module.emit_wasm()).unwrap();
    let mut store = Store::new(engine, ());
    let instance = linker.instantiate(&mut store, &module).unwrap();

    let entrypoint = instance
        .get_typed_func::<T, U>(&mut store, function_name)
        .unwrap();

    let memory = instance.get_memory(&mut store, "memory").unwrap();
    memory
        .write(
            &mut store,
            INITIAL_MEMORY_OFFSET as usize,
            &initial_memory_data,
        )
        .unwrap();

    (linker, instance, store, entrypoint)
}

pub fn get_linker_with_host_debug_functions<T>() -> Linker<T> {
    let mut linker = Linker::new(&Engine::default());
    linker
        .func_wrap("", "print_i64", |param: i64| {
            println!("--- i64 ---> {param}");
        })
        .unwrap();

    linker
        .func_wrap("", "print_i32", |param: i32| {
            println!("--- i32 ---> {param}");
        })
        .unwrap();

    linker
        .func_wrap("", "print_separator", || {
            println!("-----------------------------------------------");
        })
        .unwrap();

    linker
        .func_wrap(
            "",
            "print_memory_from",
            |mut caller: Caller<'_, T>, ptr: i32, len: i32| {
                let memory = match caller.get_export("memory") {
                    Some(wasmtime::Extern::Memory(mem)) => mem,
                    _ => panic!("failed to find host memory"),
                };

                let mut result = vec![0; len as usize];
                memory.read(&caller, ptr as usize, &mut result).unwrap();
                println!("Data {result:?}");

                println!("In chunks of 32 bytes:");
                for chunk in result.chunks(32) {
                    // print each byte in hex, for example
                    for b in chunk {
                        print!("{b:?} ");
                    }
                    println!(); // newline after each 32‑byte chunk
                }

                println!("--- --- ---\n");
            },
        )
        .unwrap();

    linker
        .func_wrap("", "print_u128", |mut caller: Caller<'_, T>, ptr: i32| {
            println!("--- u128 ---\nPointer {ptr}");

            let memory = match caller.get_export("memory") {
                Some(wasmtime::Extern::Memory(mem)) => mem,
                _ => panic!("failed to find host memory"),
            };

            let mut result = [0; 16];
            memory.read(&caller, ptr as usize, &mut result).unwrap();
            println!("Data {result:?}");
            println!("Decimal data {}", u128::from_le_bytes(result));
            println!("--- end u128 ---\n");
        })
        .unwrap();

    linker
        .func_wrap(
            "",
            "print_address",
            |mut caller: Caller<'_, T>, ptr: i32| {
                println!("--- address ---\nPointer {ptr}");

                let memory = match caller.get_export("memory") {
                    Some(wasmtime::Extern::Memory(mem)) => mem,
                    _ => panic!("failed to find host memory"),
                };

                let mut result = [0; 32];
                memory.read(&caller, ptr as usize, &mut result).unwrap();
                println!(
                    "Data 0x{}",
                    result[12..]
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<String>()
                );
                println!("--- end address ---\n");
            },
        )
        .unwrap();
    linker
}

// TODO: move this somewhere else
pub fn get_linker_with_native_keccak256<T>() -> Linker<T> {
    let mut linker = Linker::new(&Engine::default());

    // Define the native_keccak256 function
    linker
        .func_wrap(
            "vm_hooks",
            "native_keccak256",
            |mut caller: wasmtime::Caller<'_, T>,
             input_data_ptr: u32,
             data_length: u32,
             return_data_ptr: u32| {
                let memory = match caller.get_export("memory") {
                    Some(wasmtime::Extern::Memory(mem)) => mem,
                    _ => panic!("failed to find host memory"),
                };

                let mut input_data = vec![0; data_length as usize];
                memory
                    .read(&caller, input_data_ptr as usize, &mut input_data)
                    .unwrap();

                let hash = alloy_primitives::keccak256(input_data);

                memory
                    .write(&mut caller, return_data_ptr as usize, hash.as_slice())
                    .unwrap();

                Ok(())
            },
        )
        .unwrap();

    linker
}

pub fn inject_host_debug_functions(module: &mut Module) {
    let func_ty = module.types.add(&[ValType::I32], &[]);
    module.add_import_func("", "print_i32", func_ty);

    let func_ty = module.types.add(&[ValType::I64], &[]);
    module.add_import_func("", "print_i64", func_ty);

    let func_ty = module.types.add(&[ValType::I32], &[]);
    module.add_import_func("", "print_u128", func_ty);

    let func_ty = module.types.add(&[], &[]);
    module.add_import_func("", "print_separator", func_ty);
}

#[macro_export]
macro_rules! test_compilation_context {
    ($memory_id: ident, $allocator: ident, $compilation_context_globals: ident) => {
        $crate::CompilationContext {
            root_module_data: &$crate::ModuleData::default(),
            deps_data: &std::collections::HashMap::new(),
            memory_id: $memory_id,
            allocator: $allocator,
            globals: $compilation_context_globals,
            empty_signature: $crate::translation::intermediate_types::ISignature {
                arguments: Vec::new(),
                returns: Vec::new(),
            },
        }
    };
}

/// Helper to verify that a runtime error was correctly written to memory
pub fn assert_runtime_error(
    store: &mut wasmtime::Store<()>,
    instance: &wasmtime::Instance,
    error: RuntimeError,
) {
    let memory = instance.get_memory(&mut *store, "memory").unwrap();

    // Read the error pointer from the data segment
    let mut error_ptr_bytes = vec![0; 4];
    memory
        .read(
            &mut *store,
            DATA_ABORT_MESSAGE_PTR_OFFSET as usize,
            &mut error_ptr_bytes,
        )
        .unwrap();

    let error_ptr = i32::from_le_bytes(error_ptr_bytes.try_into().unwrap());

    // If the error pointer is 0, it means that no error occurred
    assert_ne!(error_ptr, 0);

    // Load the length
    let mut error_length_bytes = vec![0; 4];
    memory
        .read(&mut *store, error_ptr as usize, &mut error_length_bytes)
        .unwrap();

    let error_length = i32::from_le_bytes(error_length_bytes.try_into().unwrap());

    let mut result_data = vec![0; error_length as usize];
    memory
        .read(&mut *store, (error_ptr + 4) as usize, &mut result_data)
        .unwrap();

    let expected = error.encode_abi();
    assert_eq!(result_data, expected);
}

#[cfg(feature = "inject-host-debug-fns")]
pub fn inject_debug_fns(module: &mut walrus::Module) {
    if cfg!(feature = "inject-host-debug-fns") {
        let func_ty = module.types.add(&[ValType::I32], &[]);
        module.add_import_func("", "print_i32", func_ty);

        let func_ty = module.types.add(&[ValType::I32, ValType::I32], &[]);
        module.add_import_func("", "print_memory_from", func_ty);

        let func_ty = module.types.add(&[ValType::I64], &[]);
        module.add_import_func("", "print_i64", func_ty);

        let func_ty = module.types.add(&[ValType::I32], &[]);
        module.add_import_func("", "print_u128", func_ty);

        let func_ty = module.types.add(&[], &[]);
        module.add_import_func("", "print_separator", func_ty);

        let func_ty = module.types.add(&[ValType::I32], &[]);
        module.add_import_func("", "print_address", func_ty);
    }
}

#[cfg(feature = "inject-host-debug-fns")]
#[macro_export]
macro_rules! declare_host_debug_functions {
    ($module: ident) => {
        (
            $module.imports.get_func("", "print_i32").unwrap(),
            $module.imports.get_func("", "print_i64").unwrap(),
            $module.imports.get_func("", "print_memory_from").unwrap(),
            $module.imports.get_func("", "print_address").unwrap(),
            $module.imports.get_func("", "print_separator").unwrap(),
            $module.imports.get_func("", "print_u128").unwrap(),
        )
    };
}
