use walrus::Module;
use wasmtime::{Caller, Engine, Extern, Linker, Module as WasmModule, Store};

struct ModuleData {
    pub data: Vec<u8>,
    pub return_data: Vec<u8>,
}

pub struct RuntimeSandbox {
    engine: Engine,
    linker: Linker<ModuleData>,
    module: WasmModule,
}

impl RuntimeSandbox {
    pub fn new(module: &mut Module) -> Self {
        let engine = Engine::default();
        let module = WasmModule::from_binary(&engine, &module.emit_wasm()).unwrap();

        let mut linker = Linker::new(&engine);

        let mem_export = module.get_export_index("memory").unwrap();

        linker
            .func_wrap(
                "vm_hooks",
                "read_args",
                move |mut caller: Caller<'_, ModuleData>, args_ptr: u32| {
                    let mem = match caller.get_module_export(&mem_export) {
                        Some(Extern::Memory(mem)) => mem,
                        _ => panic!("failed to find host memory"),
                    };

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
                      _return_data_pointer: u32,
                      _return_data_length: u32| {
                    let mem = match caller.get_module_export(&mem_export) {
                        Some(Extern::Memory(mem)) => mem,
                        _ => panic!("failed to find host memory"),
                    };

                    let mut result = vec![0; _return_data_length as usize];
                    mem.read(&caller, _return_data_pointer as usize, &mut result)
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

        Self {
            engine,
            linker,
            module,
        }
    }

    /// Crates a temporary runtime sandbox instance and calls the entrypoint with the given data.
    ///
    /// Returns the result of the entrypoint call and the return data.
    pub fn call_entrypoint(&self, data: Vec<u8>) -> (i32, Vec<u8>) {
        let data_len = data.len() as i32;
        let mut store = Store::new(
            &self.engine,
            ModuleData {
                data,
                return_data: vec![],
            },
        );
        let instance = self.linker.instantiate(&mut store, &self.module).unwrap();

        let entrypoint = instance
            .get_typed_func::<i32, i32>(&mut store, "user_entrypoint")
            .unwrap();

        let result = entrypoint.call(&mut store, data_len).unwrap();

        (result, store.data().return_data.clone())
    }
}
