use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use move_bytecode_to_wasm::translate_single_module;
use move_package::{BuildConfig, LintFlag, source_package::layout::SourcePackageLayout};
use walrus::Module;
use wasmtime::{Caller, Engine, Extern, Linker, Memory, Module as WasmModule, Store, TypedFunc};

pub struct ModuleData {
    pub data: Vec<u8>,
    pub return_data: Vec<u8>,
}

pub fn setup_wasmtime_module(
    module: &mut Module,
    data: ModuleData,
) -> (Memory, Store<ModuleData>, TypedFunc<i32, i32>) {
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

    let mut store = Store::new(&engine, data);
    let instance = linker.instantiate(&mut store, &module).unwrap();

    let entrypoint = instance
        .get_typed_func::<i32, i32>(&mut store, "user_entrypoint")
        .unwrap();

    let memory = instance
        .get_memory(&mut store, "memory")
        .expect("failed to find `memory` export");

    (memory, store, entrypoint)
}

pub fn reroot_path(path: Option<&Path>) -> anyhow::Result<PathBuf> {
    let path = path
        .map(Path::canonicalize)
        .unwrap_or_else(|| PathBuf::from(".").canonicalize())?;
    // Always root ourselves to the package root, and then compile relative to that.
    let rooted_path = SourcePackageLayout::try_find_root(&path)?;
    std::env::set_current_dir(rooted_path).unwrap();

    Ok(PathBuf::from("."))
}

pub fn translate_test_package(path: &str, module_name: &str) -> Module {
    let temp_install_directory = std::env::temp_dir().join("move-bytecode-to-wasm");

    let build_config = BuildConfig {
        dev_mode: false,
        test_mode: false,
        generate_docs: false,
        save_disassembly: false,
        install_dir: Some(temp_install_directory),
        force_recompilation: false,
        lock_file: None,
        fetch_deps_only: false,
        skip_fetch_latest_git_deps: true,
        default_flavor: None,
        default_edition: None,
        deps_as_root: false,
        silence_warnings: false,
        warnings_are_errors: false,
        additional_named_addresses: BTreeMap::new(),
        implicit_dependencies: BTreeMap::new(),
        json_errors: false,
        lint_flag: LintFlag::default(),
    };

    let path = Path::new(path);

    let rerooted_path = reroot_path(Some(path)).unwrap();
    let package = build_config
        .compile_package(&rerooted_path, &mut Vec::new())
        .unwrap();

    translate_single_module(&package, module_name)
}
