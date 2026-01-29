// Copyright (c) The Move Contributors
// Copyright (c) 2025 Rather Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

pub mod activate;
pub mod build;
pub mod deploy;
pub mod disassemble;
pub mod docgen;
pub mod export_abi;
pub mod info;
pub mod new;
pub mod test;

use move_bytecode_to_wasm::{
    error::{CompilationError, ICEError, ICEErrorKind},
    translate_package,
};
use move_package::{
    compilation::compiled_package::CompiledPackage, source_package::layout::SourcePackageLayout,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
};

pub fn reroot_path(path: Option<&Path>) -> anyhow::Result<PathBuf> {
    let path = path
        .map(Path::canonicalize)
        .unwrap_or_else(|| PathBuf::from(".").canonicalize())?;
    // Always root ourselves to the package root, and then compile relative to that.
    let rooted_path = SourcePackageLayout::try_find_root(&path)?;
    std::env::set_current_dir(rooted_path).unwrap();

    Ok(PathBuf::from("."))
}

pub fn translate_package_cli(
    package: &CompiledPackage,
    rerooted_path: &Path,
    install_dir: Option<PathBuf>,
    emit_wat: bool,
    verbose: bool,
    optimize: bool,
    test_mode: bool,
) -> Result<(), Box<CompilationError>> {
    let build_directory = get_build_directory(rerooted_path, package, &install_dir);

    // Create the build directory if it doesn't exist
    std::fs::create_dir_all(&build_directory)
        .map_err(|e| ICEError::new(ICEErrorKind::Unexpected(e.into())))?;

    let mut modules_data = HashMap::new();

    let mut modules = translate_package(package, None, &mut modules_data, verbose, test_mode)?;

    for (module_name, module) in modules.iter_mut() {
        let wasm_file_path = build_directory.join(format!("{module_name}.wasm"));
        module
            .emit_wasm_file(&wasm_file_path)
            .map_err(|e| ICEError::new(ICEErrorKind::Unexpected(e.into())))?;

        if optimize {
            let mut optimizations =
                wasm_opt::OptimizationOptions::new_optimize_for_size_aggressively();
            optimizations
                .optimize_level(wasm_opt::OptimizeLevel::Level3)
                .add_pass(wasm_opt::Pass::StripDebug)
                .add_pass(wasm_opt::Pass::StripDwarf)
                .add_pass(wasm_opt::Pass::InliningOptimizing)
                .add_pass(wasm_opt::Pass::Inlining)
                .add_pass(wasm_opt::Pass::CoalesceLocals)
                .add_pass(wasm_opt::Pass::CodeFolding)
                .add_pass(wasm_opt::Pass::Directize)
                .add_pass(wasm_opt::Pass::Dce)
                .add_pass(wasm_opt::Pass::Vacuum)
                .shrink_level(wasm_opt::ShrinkLevel::Level2)
                .enable_feature(wasm_opt::Feature::BulkMemory)
                .disable_feature(wasm_opt::Feature::Simd)
                .disable_feature(wasm_opt::Feature::Multivalue);

            optimizations.run(&wasm_file_path, &wasm_file_path).unwrap();
        }

        if emit_wat {
            // Convert to WAT format
            let wat = wasmprinter::print_bytes(module.emit_wasm())
                .map_err(|e| ICEError::new(ICEErrorKind::Unexpected(e.into())))?;
            std::fs::write(
                build_directory.join(format!("{module_name}.wat")),
                wat.as_bytes(),
            )
            .map_err(|e| ICEError::new(ICEErrorKind::Io(e)))?;
        }
    }

    Ok(())
}

pub fn get_build_directory(
    rerooted_path: &Path,
    package: &CompiledPackage,
    install_dir: &Option<PathBuf>,
) -> PathBuf {
    if let Some(install_dir) = install_dir {
        install_dir.join(format!(
            "build/{}/wasm",
            package.compiled_package_info.package_name
        ))
    } else {
        rerooted_path.join(format!(
            "build/{}/wasm",
            package.compiled_package_info.package_name
        ))
    }
}

pub fn cargo_stylus_installed() -> bool {
    Command::new("sh")
        .arg("-c")
        .arg("command -v cargo-stylus > /dev/null")
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
