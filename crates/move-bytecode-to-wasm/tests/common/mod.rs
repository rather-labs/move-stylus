use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock, Mutex},
};

use move_bytecode_to_wasm::{
    compilation_context::{ModuleData, ModuleId},
    translate_package, translate_single_module,
};
use move_package::{BuildConfig, LintFlag, compilation::compiled_package::CompiledPackage};
use move_packages_build::implicit_dependencies;
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::fixture;
use walrus::Module;

type ModuleCache = LazyLock<Mutex<HashMap<(&'static str, String), Arc<Vec<u8>>>>>;

type ModuleDependenciesCache<'move_module> =
    LazyLock<Mutex<HashMap<ModuleId, ModuleData<'move_module>>>>;

/// This will be used to avoud recompiling test files multiple times
static MODULE_CACHE: ModuleCache = LazyLock::new(|| Mutex::new(HashMap::new()));

// This is the cache for all the dependencies the modules might have
static MODULE_DEPENDENCIES_CACHE: ModuleDependenciesCache =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry_result in fs::read_dir(src)? {
        let entry = entry_result?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else if file_type.is_file() {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

pub fn reroot_path(path: &Path) -> PathBuf {
    // Copy files to temp to avoid file locks
    // Use a more unique identifier to prevent conflicts between concurrent tests
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    // Use a random number instead of thread ID for uniqueness
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    thread::current().id().hash(&mut hasher);
    let thread_hash = hasher.finish();

    let temp_install_directory = std::env::temp_dir()
        .join("move-bytecode-to-wasm")
        //.join(format!("{}", path.file_name().unwrap().to_string_lossy(),));
        .join(format!(
            "{}_{}_{}",
            timestamp,
            thread_hash,
            path.file_name().unwrap().to_string_lossy(),
        ));

    // copy source file to dir
    let _ = fs::create_dir_all(temp_install_directory.join("sources"));

    // If the path is a directory, we copy all the move files to the temp dir
    if path.is_dir() {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let filepath = entry.path();
            if filepath.is_file() {
                fs::copy(
                    &filepath,
                    temp_install_directory
                        .join("sources")
                        .join(filepath.file_name().unwrap()),
                )
                .unwrap();
            }
        }
    } else {
        std::fs::copy(
            path,
            temp_install_directory
                .join("sources")
                .join(path.file_name().unwrap()),
        )
        .unwrap();
    }

    temp_install_directory
}

fn create_move_toml_with_framework(install_dir: &Path, framework_dir: &str) {
    copy_dir_recursive(
        &PathBuf::from(framework_dir),
        &install_dir.join("stylus-framework"),
    )
    .unwrap();

    // create Move.toml in dir
    std::fs::write(
        install_dir.join("Move.toml"),
        r#"[package]
name = "test"
edition = "2024"

[addresses]
test = "0x0"

[dependencies]
StylusFramework = { local = "./stylus-framework/" }
"#,
    )
    .unwrap();
}

fn get_build_config() -> BuildConfig {
    BuildConfig {
        dev_mode: false,
        test_mode: false,
        generate_docs: false,
        save_disassembly: false,
        install_dir: None,
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
        implicit_dependencies: implicit_dependencies(),
        json_errors: false,
        lint_flag: LintFlag::default(),
    }
}

#[allow(dead_code)]
/// Translates a complete package. It outputs all the corresponding wasm modules
pub fn translate_test_package(path: &'static str, module_name: &str) -> Arc<Vec<u8>> {
    let mut cache = MODULE_CACHE.lock().unwrap();
    if let Some(cached_module) = cache.get(&(path, module_name.to_owned())) {
        // println!("CACHE HIT for {}::{}", path, module_name);
        return cached_module.clone();
    }

    // println!("CACHE MISS for {}::{}", path, module_name);

    let mut dependencies_cache = MODULE_DEPENDENCIES_CACHE.lock().unwrap();

    let rerooted_path = reroot_path(Path::new(path));
    create_move_toml_with_framework(&rerooted_path, "../../stylus-framework");

    let package = match get_build_config().compile_package(&rerooted_path, &mut Vec::new()) {
        Ok(pkg) => pkg,
        Err(err) => {
            drop(cache);
            drop(dependencies_cache);
            panic!(
                "Failed to compile package at path: {}\nError: {:?}",
                rerooted_path.display(),
                err
            );
        }
    };
    let package: &'static CompiledPackage = Box::leak(Box::new(package));

    // println!("Translating package at path: {}", rerooted_path.display());
    let compiled_modules = match translate_package(package, None, &mut dependencies_cache, false) {
        Ok(modules) => modules,
        Err(err) => {
            drop(cache);
            drop(dependencies_cache);
            panic!(
                "Failed to translate package at path: {}\nError: {:?}",
                rerooted_path.display(),
                err
            );
        }
    };

    for (module_name, mut module) in compiled_modules.into_iter() {
        // println!("CACHE INSERT for {}::{}", path, module_name);
        cache.insert((path, module_name), Arc::new(module.emit_wasm()));
    }

    match cache.get(&(path, module_name.to_owned())) {
        None => {
            drop(cache);
            drop(dependencies_cache);
            panic!(
                "Module {} not found in package at path: {}. Is it named worng in the test?",
                module_name,
                rerooted_path.display()
            );
        }
        Some(module) => module.clone(),
    }
}

#[allow(dead_code)]
/// Translates a single test module and returns Result for error checking
pub fn translate_test_package_with_framework_result(
    path: &str,
    module_name: &str,
) -> Result<Module, move_bytecode_to_wasm::error::CompilationError> {
    let path = Path::new(path);
    let rerooted_path = reroot_path(path);
    create_move_toml_with_framework(&rerooted_path, "../../stylus-framework");

    let mut dependencies_cache = MODULE_DEPENDENCIES_CACHE.lock().unwrap();

    let package = get_build_config()
        .compile_package(&rerooted_path, &mut Vec::new())
        .unwrap();
    let package: &'static CompiledPackage = Box::leak(Box::new(package));

    translate_single_module(package, module_name, &mut dependencies_cache)
}

#[allow(dead_code)]
pub fn run_test(
    runtime: &crate::common::RuntimeSandbox,
    call_data: Vec<u8>,
    expected_result: Vec<u8>,
) -> Result<(), anyhow::Error> {
    let (result, return_data) = runtime.call_entrypoint(call_data)?;
    anyhow::ensure!(
        result == 0,
        "Function returned non-zero exit code: {result}"
    );
    anyhow::ensure!(
        return_data == expected_result,
        "return data mismatch:\nreturned:{return_data:?}\nexpected:{expected_result:?}"
    );

    Ok(())
}

#[macro_export]
macro_rules! declare_fixture {
    ($module_name:literal, $source_path:literal) => {
        #[fixture]
        #[once]
        pub fn runtime() -> move_test_runner::wasm_runner::RuntimeSandbox {
            let translated_package =
                $crate::common::translate_test_package($source_path, $module_name);

            move_test_runner::wasm_runner::RuntimeSandbox::from_binary(&translated_package)
        }
    };
}

#[fixture]
pub fn runtime(
    #[default("")] module_name: &str,
    #[default("")] source_path: &'static str,
) -> RuntimeSandbox {
    let translated_package = translate_test_package(source_path, module_name);

    RuntimeSandbox::from_binary(&translated_package)
}
