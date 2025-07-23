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

    temp_install_directory
}

fn create_move_toml(install_dir: &Path) {
    // create Move.toml in dir
    std::fs::write(
        install_dir.join("Move.toml"),
        r#"[package]
name = "test"
edition = "2024"

[addresses]
test = "0x0"
"#,
    )
    .unwrap();
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

fn get_build_confing() -> BuildConfig {
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

// TODO: rename to translate_test_module
#[allow(dead_code)]
/// Translates a single test module
pub fn translate_test_package(path: &str, module_name: &str) -> Module {
    let path = Path::new(path);
    let rerooted_path = reroot_path(path);
    create_move_toml(&rerooted_path);

    let package = get_build_confing()
        .compile_package(&rerooted_path, &mut Vec::new())
        .unwrap();

    translate_single_module(package, module_name)
}

// TODO: rename to translate_test_complete_package when translate_test_package is renamed
#[allow(dead_code)]
/// Translates a complete package. It outputs all the corresponding wasm modules
pub fn translate_test_complete_package(path: &str) -> HashMap<String, Module> {
    let path = Path::new(path);

    let rerooted_path = reroot_path(path);
    create_move_toml(&rerooted_path);

    std::env::set_current_dir(&rerooted_path).unwrap();
    let package = get_build_confing()
        .compile_package(&rerooted_path, &mut Vec::new())
        .unwrap();

    translate_package(package, None)
}

#[allow(dead_code)]
/// Translates a single test module
pub fn translate_test_package_with_framework(path: &str, module_name: &str) -> Module {
    let path = Path::new(path);
    let rerooted_path = reroot_path(path);
    create_move_toml_with_framework(&rerooted_path, "../../stylus-framework");

    let package = get_build_confing()
        .compile_package(&rerooted_path, &mut Vec::new())
        .unwrap();

    translate_single_module(package, module_name)
}
