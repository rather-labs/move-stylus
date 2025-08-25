#![allow(dead_code)]
pub mod constants;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, mpsc},
};

use alloy_primitives::keccak256;
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
    pub log_events: Arc<Mutex<mpsc::Receiver<Vec<u8>>>>,
    current_tx_origin: Arc<Mutex<[u8; 20]>>,
    current_msg_sender: Arc<Mutex<[u8; 20]>>,
    storage: Arc<Mutex<HashMap<[u8; 32], [u8; 32]>>>,
}

macro_rules! link_fn_ret_constant {
    ($linker:expr, $name:literal, $constant:expr, $constant_type: ty) => {
        $linker
            .func_wrap(
                "vm_hooks",
                $name,
                move |_caller: Caller<'_, ModuleData>| -> $constant_type {
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

        let storage: Arc<Mutex<HashMap<[u8; 32], [u8; 32]>>> = Arc::new(Mutex::new(HashMap::new()));
        let current_tx_origin = Arc::new(Mutex::new(SIGNER_ADDRESS));
        let current_msg_sender = Arc::new(Mutex::new(MSG_SENDER_ADDRESS));

        let (log_sender, log_receiver) = mpsc::channel::<Vec<u8>>();
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
                "native_keccak256",
                move |mut caller: Caller<'_, ModuleData>,
                      input_data_ptr: u32,
                      data_length: u32,
                      return_data_ptr: u32| {
                    let mem = match caller.get_module_export(&mem_export) {
                        Some(Extern::Memory(mem)) => mem,
                        _ => panic!("failed to find host memory"),
                    };

                    let mut input_data = vec![0; data_length as usize];
                    mem.read(&caller, input_data_ptr as usize, &mut input_data)
                        .unwrap();

                    let hash = keccak256(input_data);

                    mem.write(&mut caller, return_data_ptr as usize, hash.as_slice())
                        .unwrap();

                    Ok(())
                },
            )
            .unwrap();

        linker
            .func_wrap(
                "vm_hooks",
                "emit_log",
                move |mut caller: Caller<'_, ModuleData>, ptr: u32, len: u32, _topic: u32| {
                    let mem = get_memory(&mut caller);
                    let mut buffer = vec![0; len as usize];

                    mem.read(&mut caller, ptr as usize, &mut buffer).unwrap();

                    log_sender.send(buffer.to_vec()).unwrap();
                },
            )
            .unwrap();

        let storage_for_cache = storage.clone();
        linker
            .func_wrap(
                "vm_hooks",
                "storage_cache_bytes32",
                move |mut caller: Caller<'_, ModuleData>, key_ptr: u32, value_ptr: u32| {
                    let mem = get_memory(&mut caller);
                    let mut key_buffer = [0; 32];
                    mem.read(&mut caller, key_ptr as usize, &mut key_buffer)
                        .unwrap();

                    let mut value_buffer = [0; 32];
                    mem.read(&mut caller, value_ptr as usize, &mut value_buffer)
                        .unwrap();

                    let mut storage = storage_for_cache.lock().unwrap();
                    (*storage).insert(key_buffer, value_buffer);
                },
            )
            .unwrap();

        let storage_for_cache = storage.clone();
        linker
            .func_wrap(
                "vm_hooks",
                "storage_load_bytes32",
                move |mut caller: Caller<'_, ModuleData>, key_ptr: u32, dest_ptr: u32| {
                    let mem = get_memory(&mut caller);
                    let mut key_buffer = [0; 32];
                    mem.read(&mut caller, key_ptr as usize, &mut key_buffer)
                        .unwrap();

                    let storage = storage_for_cache.lock().unwrap();
                    let value = (*storage).get(&key_buffer).unwrap_or(&[0; 32]);

                    mem.write(&mut caller, dest_ptr as usize, value.as_slice())
                        .unwrap();
                },
            )
            .unwrap();

        let tx_orign = current_tx_origin.clone();
        linker
            .func_wrap(
                "vm_hooks",
                "tx_origin",
                move |mut caller: Caller<'_, ModuleData>, ptr: u32| {
                    let mem = match caller.get_export("memory") {
                        Some(Extern::Memory(mem)) => mem,
                        _ => panic!("failed to find host memory"),
                    };

                    let data = tx_orign.lock().unwrap();
                    mem.write(&mut caller, ptr as usize, &*data).unwrap();
                },
            )
            .unwrap();

        let msg_sender = current_msg_sender.clone();
        linker
            .func_wrap(
                "vm_hooks",
                "msg_sender",
                move |mut caller: Caller<'_, ModuleData>, ptr: u32| {
                    let mem = match caller.get_export("memory") {
                        Some(Extern::Memory(mem)) => mem,
                        _ => panic!("failed to find host memory"),
                    };

                    let data = msg_sender.lock().unwrap();
                    mem.write(&mut caller, ptr as usize, &*data).unwrap();
                },
            )
            .unwrap();

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
            log_events: Arc::new(Mutex::new(log_receiver)),
            current_tx_origin,
            current_msg_sender,
            storage,
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

    pub fn set_tx_origin(&self, new_address: [u8; 20]) {
        *self.current_tx_origin.lock().unwrap() = new_address;
    }

    pub fn get_tx_origin(&self) -> [u8; 20] {
        *self.current_tx_origin.lock().unwrap()
    }

    pub fn set_msg_sender(&self, new_address: [u8; 20]) {
        *self.current_msg_sender.lock().unwrap() = new_address;
    }

    pub fn get_storage_at_slot(&self, slot: [u8; 32]) -> [u8; 32] {
        let storage = self.storage.lock().unwrap();
        println!("{:?}", storage);
        *storage.get(&slot).unwrap()
    }
}
