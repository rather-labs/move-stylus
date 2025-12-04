// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod abi_generate;
pub mod build;
pub mod deploy;
pub mod disassemble;
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
use std::path::{Path, PathBuf};

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
    package: CompiledPackage,
    rerooted_path: &Path,
    install_dir: Option<PathBuf>,
    emit_wat: bool,
    verbose: bool,
) -> Result<(), Box<CompilationError>> {
    let build_directory = get_build_directory(rerooted_path, &package, &install_dir);

    // Create the build directory if it doesn't exist
    std::fs::create_dir_all(&build_directory)
        .map_err(|e| ICEError::new(ICEErrorKind::Unexpected(e.into())))?;

    let mut modules = translate_package(package, None, verbose)?;

    for (module_name, module) in modules.iter_mut() {
        module
            .emit_wasm_file(build_directory.join(format!("{module_name}.wasm")))
            .map_err(|e| ICEError::new(ICEErrorKind::Unexpected(e.into())))?;

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
