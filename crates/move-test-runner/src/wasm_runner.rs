use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex, atomic::AtomicBool, mpsc},
};

use crate::constants::{
    BLOCK_BASEFEE, BLOCK_GAS_LIMIT, BLOCK_NUMBER, BLOCK_TIMESTAMP, CHAIN_ID, GAS_PRICE,
    MSG_SENDER_ADDRESS, MSG_VALUE, SIGNER_ADDRESS,
};
use alloy_primitives::{U256, keccak256};
use anyhow::Result;
// use walrus::Module;
use wasmtime::{Caller, Engine, Extern, Linker, Module as WasmModule, Store};

pub struct ModuleData {
    pub data: Vec<u8>,
    pub return_data: Vec<u8>,
}

#[allow(dead_code)]
pub struct ExecutionData {
    pub return_data: Vec<u8>,
    pub instance: wasmtime::Instance,
    pub store: Store<ModuleData>,
}

type LogEventReceiver = Arc<Mutex<mpsc::Receiver<(u32, Vec<u8>)>>>;
type CrossCrontractExecutionReceiver = Arc<Mutex<mpsc::Receiver<CrossContractExecutionData>>>;

#[allow(dead_code)]
pub struct RuntimeSandbox {
    engine: Engine,
    linker: Linker<ModuleData>,
    module: WasmModule,
    pub log_events: LogEventReceiver,
    pub cross_contract_calls: CrossCrontractExecutionReceiver,
    current_tx_origin: Arc<Mutex<[u8; 20]>>,
    current_msg_sender: Arc<Mutex<[u8; 20]>>,
    storage: Arc<Mutex<HashMap<[u8; 32], [u8; 32]>>>,
    cross_contract_call_return_data: Arc<Mutex<Vec<u8>>>,
    cross_contract_call_succeed: Arc<AtomicBool>,
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

pub enum CrossContractCallType {
    Call,
    StaticCall,
    DelegateCall,
}

#[allow(dead_code)]
pub struct CrossContractExecutionData {
    pub calldata: Vec<u8>,
    pub address: [u8; 20],
    pub gas: u64,
    pub value: U256,
    pub return_datan_len: u32,
    pub call_type: CrossContractCallType,
}

impl RuntimeSandbox {
    pub fn new(compiled_module_path: &Path) -> Self {
        let engine = Engine::default();

        let module = WasmModule::from_file(&engine, compiled_module_path).unwrap();

        let storage: Arc<Mutex<HashMap<[u8; 32], [u8; 32]>>> = Arc::new(Mutex::new(HashMap::new()));
        let current_tx_origin = Arc::new(Mutex::new(SIGNER_ADDRESS));
        let current_msg_sender = Arc::new(Mutex::new(MSG_SENDER_ADDRESS));
        let cross_contract_call_return_data = Arc::new(Mutex::new(vec![]));
        let cross_contract_call_succeed = Arc::new(AtomicBool::new(true));

        let (log_sender, log_receiver) = mpsc::channel::<(u32, Vec<u8>)>();
        let (cce_sender, cce_receiver) = mpsc::channel::<CrossContractExecutionData>();

        let mut linker = Linker::new(&engine);

        let mem_export = module.get_export_index("memory").unwrap();
        let get_memory = move |caller: &mut Caller<'_, ModuleData>| match caller
            .get_module_export(&mem_export)
        {
            Some(Extern::Memory(mem)) => mem,
            _ => panic!("failed to find host memory"),
        };

        let cccrd = cross_contract_call_return_data.clone();
        linker
            .func_wrap(
                "vm_hooks",
                "read_return_data",
                move |mut caller: Caller<'_, ModuleData>,
                      dest_ptr: u32,
                      offset: u32,
                      _size: u32| {
                    let mem = get_memory(&mut caller);

                    let data = cccrd.lock().unwrap();
                    mem.write(
                        &mut caller,
                        dest_ptr as usize + offset as usize,
                        data.as_slice(),
                    )
                    .unwrap();

                    Ok(data.as_slice().len() as u32)
                },
            )
            .unwrap();

        let cccrd = cross_contract_call_return_data.clone();
        let cccs = cross_contract_call_succeed.clone();
        let cces = cce_sender.clone();
        linker
            .func_wrap(
                "vm_hooks",
                "delegate_call_contract",
                move |mut caller: Caller<'_, ModuleData>,
                      address_ptr: u32,
                      calldata_ptr: u32,
                      calldata_len_ptr: u32,
                      gas: u64,
                      return_data_len_ptr: u32| {
                    if cccs.load(std::sync::atomic::Ordering::Relaxed) {
                        let mem = get_memory(&mut caller);

                        let mut address = [0; 20];
                        mem.read(&caller, address_ptr as usize, &mut address)
                            .unwrap();

                        let mut calldata = vec![0; calldata_len_ptr as usize];
                        mem.read(&caller, calldata_ptr as usize, &mut calldata)
                            .unwrap();

                        let cross_contract_call_return_data_len =
                            &cccrd.lock().unwrap().len().to_le_bytes()[..4];
                        mem.write(
                            &mut caller,
                            return_data_len_ptr as usize,
                            cross_contract_call_return_data_len,
                        )
                        .unwrap();

                        cces.send(CrossContractExecutionData {
                            calldata,
                            address,
                            gas,
                            value: U256::from(0),
                            return_datan_len: return_data_len_ptr,
                            call_type: CrossContractCallType::DelegateCall,
                        })
                        .unwrap();

                        Ok(0)
                    } else {
                        Ok(1)
                    }
                },
            )
            .unwrap();

        let cccrd = cross_contract_call_return_data.clone();
        let cccs = cross_contract_call_succeed.clone();
        let cces = cce_sender.clone();
        linker
            .func_wrap(
                "vm_hooks",
                "static_call_contract",
                move |mut caller: Caller<'_, ModuleData>,
                      address_ptr: u32,
                      calldata_ptr: u32,
                      calldata_len_ptr: u32,
                      gas: u64,
                      return_data_len_ptr: u32| {
                    if cccs.load(std::sync::atomic::Ordering::Relaxed) {
                        let mem = get_memory(&mut caller);

                        let mut address = [0; 20];
                        mem.read(&caller, address_ptr as usize, &mut address)
                            .unwrap();

                        let mut calldata = vec![0; calldata_len_ptr as usize];
                        mem.read(&caller, calldata_ptr as usize, &mut calldata)
                            .unwrap();

                        let cross_contract_call_return_data_len =
                            &cccrd.lock().unwrap().len().to_le_bytes()[..4];
                        mem.write(
                            &mut caller,
                            return_data_len_ptr as usize,
                            cross_contract_call_return_data_len,
                        )
                        .unwrap();

                        cces.send(CrossContractExecutionData {
                            calldata,
                            address,
                            gas,
                            value: U256::from(0),
                            return_datan_len: return_data_len_ptr,
                            call_type: CrossContractCallType::StaticCall,
                        })
                        .unwrap();

                        Ok(0)
                    } else {
                        Ok(1)
                    }
                },
            )
            .unwrap();

        let cccrd = cross_contract_call_return_data.clone();
        let cccs = cross_contract_call_succeed.clone();
        let cces = cce_sender.clone();
        linker
            .func_wrap(
                "vm_hooks",
                "call_contract",
                move |mut caller: Caller<'_, ModuleData>,
                      address_ptr: u32,
                      calldata_ptr: u32,
                      calldata_len_ptr: u32,
                      value_ptr: u32,
                      gas: u64,
                      return_data_len_ptr: u32| {
                    if cccs.load(std::sync::atomic::Ordering::Relaxed) {
                        let mem = get_memory(&mut caller);

                        let mut address = [0; 20];
                        mem.read(&caller, address_ptr as usize, &mut address)
                            .unwrap();

                        let mut calldata = vec![0; calldata_len_ptr as usize];
                        mem.read(&caller, calldata_ptr as usize, &mut calldata)
                            .unwrap();

                        let mut value = [0; 32];
                        mem.read(&caller, value_ptr as usize, &mut value).unwrap();
                        let value = U256::from_be_bytes(value);

                        let cross_contract_call_return_data_len =
                            &cccrd.lock().unwrap().len().to_le_bytes()[..4];
                        mem.write(
                            &mut caller,
                            return_data_len_ptr as usize,
                            cross_contract_call_return_data_len,
                        )
                        .unwrap();

                        cces.send(CrossContractExecutionData {
                            calldata,
                            address,
                            gas,
                            value,
                            return_datan_len: return_data_len_ptr,
                            call_type: CrossContractCallType::Call,
                        })
                        .unwrap();

                        Ok(0)
                    } else {
                        Ok(1)
                    }
                },
            )
            .unwrap();

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
                move |mut caller: Caller<'_, ModuleData>, ptr: u32, len: u32, topic: u32| {
                    let mem = get_memory(&mut caller);
                    let mut buffer = vec![0; len as usize];

                    mem.read(&mut caller, ptr as usize, &mut buffer).unwrap();

                    log_sender.send((topic, buffer.to_vec())).unwrap();
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

        Self {
            engine,
            linker,
            module,
            log_events: Arc::new(Mutex::new(log_receiver)),
            cross_contract_calls: Arc::new(Mutex::new(cce_receiver)),
            current_tx_origin,
            current_msg_sender,
            storage,
            cross_contract_call_return_data,
            cross_contract_call_succeed,
        }
    }

    /// Crates a temporary runtime sandbox instance and calls the entrypoint with the given data.
    ///
    /// Returns the result of the entrypoint call and the return data.
    pub fn call_test_function(&self, function_name: &str) -> Result<ExecutionData> {
        let data = vec![];
        let mut store = Store::new(
            &self.engine,
            ModuleData {
                data,
                return_data: vec![],
            },
        );
        let instance = self.linker.instantiate(&mut store, &self.module)?;

        let entrypoint = instance.get_func(&mut store, function_name).unwrap();

        entrypoint
            .call(&mut store, &[], &mut [])
            .map_err(|e| anyhow::anyhow!("error calling entrypoint: {e:?}"))?;

        Ok(ExecutionData {
            // result,
            return_data: store.data().return_data.clone(),
            instance,
            store,
        })
    }
}
