use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::{Path, PathBuf},
};

use move_bytecode_to_wasm::{translate_package, translate_single_module};
use move_package::{BuildConfig, LintFlag};
use move_packages_build::implicit_dependencies;
use walrus::Module;

pub mod runtime_sandbox;

pub fn reroot_path(path: &Path) -> PathBuf {
    // Copy files to temp to avoid file locks
    let temp_install_directory = std::env::temp_dir()
        .join("move-bytecode-to-wasm")
        .join(path);

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

    // create Move.toml in dir
    std::fs::write(
        temp_install_directory.join("Move.toml"),
        r#"[package]
name = "test"
edition = "2024"

[addresses]
test = "0x0"
"#,
    )
    .unwrap();

    temp_install_directory
}

// TODO: rename to translate_test_module
#[allow(dead_code)]
/// Translates a single test module
pub fn translate_test_package(path: &str, module_name: &str) -> Module {
    let build_config = BuildConfig {
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
    };

    let path = Path::new(path);
    let rerooted_path = reroot_path(path);

    let package = build_config
        .compile_package(&rerooted_path, &mut Vec::new())
        .unwrap();

    translate_single_module(package, module_name)
}

// TODO: rename to translate_test_complete_package when translate_test_package is renamed
#[allow(dead_code)]
/// Translates a complete package. It outputs all the corresponding wasm modules
pub fn translate_test_complete_package(path: &str) -> HashMap<String, Module> {
    let build_config = BuildConfig {
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
    };

    let path = Path::new(path);

    let rerooted_path = reroot_path(path);
    std::env::set_current_dir(&rerooted_path).unwrap();
    let package = build_config
        .compile_package(&rerooted_path, &mut Vec::new())
        .unwrap();

    translate_package(package, None)
}
