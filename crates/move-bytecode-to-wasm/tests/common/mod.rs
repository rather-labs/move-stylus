use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use move_bytecode_to_wasm::translate_single_module;
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
    let _ = std::fs::create_dir_all(temp_install_directory.join("sources"));
    std::fs::copy(
        path,
        temp_install_directory
            .join("sources")
            .join(path.file_name().unwrap()),
    )
    .unwrap();

    // create Move.toml in dir
    std::fs::write(
        temp_install_directory.join("Move.toml"),
        "[package]
name = \"test_primitives\"
edition = \"2024\"

",
    )
    .unwrap();

    temp_install_directory
}

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

    translate_single_module(&package, module_name)
}
