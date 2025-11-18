use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use move_package::{BuildConfig, LintFlag};
use move_packages_build::implicit_dependencies;

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

fn reroot_path(path: &Path) -> PathBuf {
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
        .join(format!(
            "{}_{}_{}",
            path.file_name().unwrap().to_string_lossy(),
            timestamp,
            thread_hash
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

/// Compiles a Move package and generates the JSON ABI for the specified module.
/// Returns the JSON string if successful.
pub fn generate_abi(path: &str, module_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    use move_bytecode_to_wasm::package_module_data;
    use move_evm_abi_generator::generate_abi;

    let path = Path::new(path);
    let rerooted_path = reroot_path(path);

    create_move_toml_with_framework(&rerooted_path, "../../stylus-framework");

    // Compile the package
    let package = get_build_config()
        .compile_package(&rerooted_path, &mut Vec::new())
        .unwrap();

    // Get the root compiled units for the module
    let root_compiled_units: Vec<_> = package
        .root_compiled_units
        .iter()
        .filter(|unit| unit.unit.name.to_string() == module_name)
        .collect();

    if root_compiled_units.is_empty() {
        return Err("No compiled units found".into());
    }

    let package_module_data = package_module_data(&package, Some(module_name.to_string()))
        .map_err(|e| format!("Failed to get package module data: {e:?}"))?;

    let abis = generate_abi(
        &package,
        &root_compiled_units,
        &package_module_data,
        true,
        false,
    )
    .map_err(|(_mapped_files, errors)| {
        format!(
            "Failed to generate ABI: {:?}",
            errors
                .iter()
                .map(|e| format!("{e:?}"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;

    // Find the ABI for the requested module, or return the first one
    let abi = abis
        .iter()
        .find(|abi| {
            abi.file
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s == module_name)
                .unwrap_or(false)
        })
        .ok_or_else(|| format!("ABI not found for module: {module_name}"))?;

    abi.content_json
        .clone()
        .ok_or_else(|| "No JSON content in ABI".to_string())
        .map_err(|e| e.into())
}

pub fn test_generated_abi(
    json_path: &str,
    module_path: &str,
    module_name: &str,
) -> Result<(), String> {
    let actual_json = generate_abi(module_path, module_name)
        .map_err(|e| format!("Failed to generate ABI: {e}"))?;
    let expected_json = fs::read_to_string(json_path)
        .map_err(|e| format!("Failed to read expected JSON file: {e}"))?;

    let mut actual_json: serde_json::Value = serde_json::from_str(&actual_json)
        .map_err(|e| format!("Failed to parse actual JSON: {e}"))?;

    let mut expected_json: serde_json::Value = serde_json::from_str(&expected_json)
        .map_err(|e| format!("Failed to parse expected JSON: {e}"))?;

    // Sort the ABI arrays by name to make comparison order-independent
    if let (Some(actual_abi), Some(expected_abi)) = (
        actual_json.get_mut("abi").and_then(|v| v.as_array_mut()),
        expected_json.get_mut("abi").and_then(|v| v.as_array_mut()),
    ) {
        // Sort by name field (empty string if name is None or missing)
        let sort_key = |item: &serde_json::Value| -> String {
            item.get("name")
                .and_then(|n| n.as_str())
                .unwrap_or_else(|| panic!("Name is missing for item: {item:?}"))
                .to_string()
        };

        actual_abi.sort_by_key(sort_key);
        expected_abi.sort_by_key(sort_key);
    }

    if actual_json != expected_json {
        let actual_pretty =
            serde_json::to_string_pretty(&actual_json).unwrap_or_else(|_| actual_json.to_string());
        let expected_pretty = serde_json::to_string_pretty(&expected_json)
            .unwrap_or_else(|_| expected_json.to_string());

        println!("Actual JSON: {actual_pretty}");
        println!("Expected JSON: {expected_pretty}");
        
        return Err("Jsons do not match".into());
    }

    Ok(())
}
