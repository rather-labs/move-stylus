//! This module contains aux functions used in unit tests in this module
#![allow(dead_code)]
use walrus::{FunctionId, MemoryId, Module, ModuleConfig, ValType};
use wasmtime::{Caller, Engine, Instance, Linker, Module as WasmModule, Store, TypedFunc};

use crate::memory::setup_module_memory;

pub fn build_module(initial_memory_offset: Option<i32>) -> (Module, FunctionId, MemoryId) {
    let config = ModuleConfig::new();
    let mut module = Module::with_config(config);
    let (allocator_func, memory_id) = setup_module_memory(&mut module, initial_memory_offset);

    (module, allocator_func, memory_id)
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
    memory.write(&mut store, 0, &initial_memory_data).unwrap();

    // Print current memory
    let memory_data = memory.data(&mut store);
    println!(
        "Current memory: {:?}",
        memory_data.iter().take(64).collect::<Vec<_>>()
    );

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
    ($memory_id: ident, $allocator: ident) => {
        $crate::CompilationContext {
            root_module_data: &$crate::ModuleData::default(),
            deps_data: &std::collections::HashMap::new(),
            memory_id: $memory_id,
            allocator: $allocator,
        }
    };
}
