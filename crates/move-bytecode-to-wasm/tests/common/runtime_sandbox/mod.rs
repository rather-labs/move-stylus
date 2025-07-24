#![allow(dead_code)]
pub mod constants;

use anyhow::Result;
use constants::{
    BLOCK_BASEFEE, BLOCK_GAS_LIMIT, BLOCK_NUMBER, BLOCK_TIMESTAMP, CHAIN_ID, GAS_PRICE,
    MSG_SENDER_ADDRESS, MSG_VALUE, SIGNER_ADDRESS,
};
use walrus::Module;
use wasmtime::{Caller, Engine, Extern, Linker, Module as WasmModule, Store};

#[cfg(feature = "inject-host-debug-fns")]
use walrus::ValType;

struct ModuleData {
    pub data: Vec<u8>,
    pub return_data: Vec<u8>,
}

pub struct RuntimeSandbox {
    engine: Engine,
    linker: Linker<ModuleData>,
    module: WasmModule,
}

macro_rules! link_fn_ret_constant {
    ($linker:expr, $name:literal, $constant:expr, $constant_type: ty) => {
        $linker
            .func_wrap(
                "vm_hooks",
                $name,
                move |_caller: Caller<'_, ModuleData>| -> $constant_type {
                    println!("{} called", $name);
                    $constant as $constant_type
                },
            )
            .unwrap();
    };
}

macro_rules! link_fn_write_constant {
    ($linker:expr, $name:literal, $constant:expr) => {
        $linker
            .func_wrap(
                "vm_hooks",
                $name,
                move |mut caller: Caller<'_, ModuleData>, ptr: u32| {
                    println!("{} called, writing in {ptr}", $name);

                    let mem = match caller.get_export("memory") {
                        Some(Extern::Memory(mem)) => mem,
                        _ => panic!("failed to find host memory"),
                    };

                    mem.write(&mut caller, ptr as usize, &$constant).unwrap();
                },
            )
            .unwrap();
    };
    () => {};
}

impl RuntimeSandbox {
    pub fn new(module: &mut Module) -> Self {
        let engine = Engine::default();

        let module = WasmModule::from_binary(&engine, &module.emit_wasm()).unwrap();

        let mut linker = Linker::new(&engine);

        let mem_export = module.get_export_index("memory").unwrap();
        let get_memory = move |caller: &mut Caller<'_, ModuleData>| match caller
            .get_module_export(&mem_export)
        {
            Some(Extern::Memory(mem)) => mem,
            _ => panic!("failed to find host memory"),
        };

        linker
            .func_wrap(
                "vm_hooks",
                "read_args",
                move |mut caller: Caller<'_, ModuleData>, args_ptr: u32| {
                    let mem = get_memory(&mut caller);

                    let args_data = caller.data().data.clone();

                    mem.write(&mut caller, args_ptr as usize, &args_data)
                        .unwrap();

                    Ok(())
                },
            )
            .unwrap();

        linker
            .func_wrap(
                "vm_hooks",
                "write_result",
                move |mut caller: Caller<'_, ModuleData>,
                      return_data_pointer: u32,
                      return_data_length: u32| {
                    let mem = match caller.get_module_export(&mem_export) {
                        Some(Extern::Memory(mem)) => mem,
                        _ => panic!("failed to find host memory"),
                    };

                    let mut result = vec![0; return_data_length as usize];
                    mem.read(&caller, return_data_pointer as usize, &mut result)
                        .unwrap();

                    let return_data = caller.data_mut();
                    return_data.return_data = result;

                    Ok(())
                },
            )
            .unwrap();

        linker
            .func_wrap("vm_hooks", "pay_for_memory_grow", |_pages: u32| {})
            .unwrap();

        linker
            .func_wrap("vm_hooks", "storage_flush_cache", |_: i32| {})
            .unwrap();

        linker
            .func_wrap(
                "vm_hooks",
                "emit_log",
                move |mut caller: Caller<'_, ModuleData>, ptr: u32, len: u32, _topic: u32| {
                    println!("emit_log, reading from {ptr}, length: {len}");

                    let mem = get_memory(&mut caller);
                    let mut buffer = vec![0; len as usize];

                    mem.read(&mut caller, ptr as usize, &mut buffer).unwrap();

                    println!("read memory: {buffer:?}");
                },
            )
            .unwrap();

        link_fn_write_constant!(linker, "tx_origin", SIGNER_ADDRESS);
        link_fn_write_constant!(linker, "msg_sender", MSG_SENDER_ADDRESS);
        link_fn_write_constant!(linker, "msg_value", MSG_VALUE.to_le_bytes::<32>());
        link_fn_write_constant!(linker, "block_basefee", BLOCK_BASEFEE.to_le_bytes::<32>());
        link_fn_write_constant!(linker, "tx_gas_price", GAS_PRICE.to_le_bytes::<32>());

        link_fn_ret_constant!(linker, "chainid", CHAIN_ID, i64);
        link_fn_ret_constant!(linker, "block_number", BLOCK_NUMBER, i64);
        link_fn_ret_constant!(linker, "block_gas_limit", BLOCK_GAS_LIMIT, i64);
        link_fn_ret_constant!(linker, "block_timestamp", BLOCK_TIMESTAMP, i64);

        if cfg!(feature = "inject-host-debug-fns") {
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
                    "print_u128",
                    |mut caller: Caller<'_, ModuleData>, ptr: i32| {
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
                    },
                )
                .unwrap();

            linker
                .func_wrap(
                    "",
                    "print_memory_from",
                    |mut caller: Caller<'_, ModuleData>, ptr: i32| {
                        println!("--- 512 from position {ptr}----");

                        let memory = match caller.get_export("memory") {
                            Some(wasmtime::Extern::Memory(mem)) => mem,
                            _ => panic!("failed to find host memory"),
                        };

                        let mut result = [0; 512];
                        memory.read(&caller, ptr as usize, &mut result).unwrap();
                        println!("Data {result:?}");
                        println!("--- --- ---\n");
                    },
                )
                .unwrap();

            linker
                .func_wrap(
                    "",
                    "print_address",
                    |mut caller: Caller<'_, ModuleData>, ptr: i32| {
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
                                .map(|b| format!("{:02x}", b))
                                .collect::<String>()
                        );
                        println!("--- end address ---\n");
                    },
                )
                .unwrap();
        }

        Self {
            engine,
            linker,
            module,
        }
    }

    /// Crates a temporary runtime sandbox instance and calls the entrypoint with the given data.
    ///
    /// Returns the result of the entrypoint call and the return data.
    pub fn call_entrypoint(&self, data: Vec<u8>) -> Result<(i32, Vec<u8>)> {
        let data_len = data.len() as i32;
        let mut store = Store::new(
            &self.engine,
            ModuleData {
                data,
                return_data: vec![],
            },
        );
        let instance = self.linker.instantiate(&mut store, &self.module)?;

        let entrypoint = instance.get_typed_func::<i32, i32>(&mut store, "user_entrypoint")?;

        let result = entrypoint
            .call(&mut store, data_len)
            .map_err(|e| anyhow::anyhow!("error calling entrypoint: {e:?}"))?;

        Ok((result, store.data().return_data.clone()))
    }
}
