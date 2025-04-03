use walrus::{ExportItem, ImportKind, Module, ValType};

pub enum WasmValidationError {
    InvalidWasm(String),
    InvalidStylusInterface(String),
}

impl std::fmt::Debug for WasmValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WasmValidationError::InvalidWasm(s) => write!(f, "Invalid Wasm: {}", s),
            WasmValidationError::InvalidStylusInterface(s) => {
                write!(f, "Invalid Stylus Interface: {}", s)
            }
        }
    }
}

/// Validate the Wasm module
///
/// This function validates the Wasm module consistency using the wasmparser crate.
/// It also validates the stylus interface requirements.
pub fn validate_stylus_wasm(module: &mut Module) -> Result<(), WasmValidationError> {
    let mut validator = wasmparser::Validator::new();

    validator
        .validate_all(&module.emit_wasm())
        .map_err(|e| WasmValidationError::InvalidWasm(e.to_string()))?;

    validate_entrypoint_function(module)?;
    validate_memory_export(module)?;
    validate_pay_for_memory_grow_import(module)?;

    Ok(())
}

/// `user_entrypoint` is mandatory and should be of type `user_entrypoint(i32) -> i32`
fn validate_entrypoint_function(module: &mut Module) -> Result<(), WasmValidationError> {
    let user_entrypoint = module
        .exports
        .iter()
        .find(|export| export.name == "user_entrypoint")
        .ok_or(WasmValidationError::InvalidStylusInterface(
            "user_entrypoint export not found".to_string(),
        ))?;

    let ExportItem::Function(function) = user_entrypoint.item else {
        return Err(WasmValidationError::InvalidStylusInterface(
            "user_entrypoint export is not a function".to_string(),
        ));
    };

    // Validate function signature
    let function_type_id = module.funcs.get(function).ty();
    let function_type = module.types.get(function_type_id);

    if function_type.params().len() != 1 {
        return Err(WasmValidationError::InvalidStylusInterface(
            "user_entrypoint function must have one parameter".to_string(),
        ));
    }

    if function_type.params()[0] != ValType::I32 {
        return Err(WasmValidationError::InvalidStylusInterface(
            "user_entrypoint function must have i32 parameter".to_string(),
        ));
    }

    if function_type.results().len() != 1 {
        return Err(WasmValidationError::InvalidStylusInterface(
            "user_entrypoint function must have one return value".to_string(),
        ));
    }

    if function_type.results()[0] != ValType::I32 {
        return Err(WasmValidationError::InvalidStylusInterface(
            "user_entrypoint function must have i32 return value".to_string(),
        ));
    }

    Ok(())
}

/// The module should create a local memory with the following characteristics:
/// - shared: false
/// - memory64: false
/// - page_size_log2: None
/// - initial: 1
/// - maximum: None
///
/// And export it as `memory`
fn validate_memory_export(module: &mut Module) -> Result<(), WasmValidationError> {
    let memory_export = module
        .exports
        .iter()
        .find(|export| export.name == "memory")
        .ok_or(WasmValidationError::InvalidStylusInterface(
            "memory export not found".to_string(),
        ))?;

    let ExportItem::Memory(memory_id) = memory_export.item else {
        return Err(WasmValidationError::InvalidStylusInterface(
            "memory export is not a memory".to_string(),
        ));
    };

    let memory = module.memories.get(memory_id);
    if memory.shared {
        return Err(WasmValidationError::InvalidStylusInterface(
            "memory export must not be shared".to_string(),
        ));
    }
    if memory.memory64 {
        return Err(WasmValidationError::InvalidStylusInterface(
            "memory export must not be 64-bit".to_string(),
        ));
    }
    if memory.page_size_log2.is_some() {
        return Err(WasmValidationError::InvalidStylusInterface(
            "memory page_size_log2 must be None".to_string(),
        ));
    }
    if memory.initial != 1 {
        return Err(WasmValidationError::InvalidStylusInterface(
            "memory export must have an initial size of 1 page".to_string(),
        ));
    }
    if memory.maximum.is_some() {
        return Err(WasmValidationError::InvalidStylusInterface(
            "memory export must have no maximum".to_string(),
        ));
    }

    Ok(())
}

/// `pay_for_memory_grow` is mandatory and should be of type `pay_for_memory_grow(i32) -> ()`
fn validate_pay_for_memory_grow_import(module: &mut Module) -> Result<(), WasmValidationError> {
    let pay_for_memory_grow_import = module
        .imports
        .iter()
        .find(|import| import.name == "pay_for_memory_grow")
        .ok_or(WasmValidationError::InvalidStylusInterface(
            "pay_for_memory_grow import not found".to_string(),
        ))?;

    if pay_for_memory_grow_import.module != "vm_hooks" {
        return Err(WasmValidationError::InvalidStylusInterface(
            "pay_for_memory_grow import must be from vm_hooks".to_string(),
        ));
    }

    let ImportKind::Function(function_type_id) =
        module.imports.get(pay_for_memory_grow_import.id()).kind
    else {
        return Err(WasmValidationError::InvalidStylusInterface(
            "pay_for_memory_grow import is not a function".to_string(),
        ));
    };

    let function_type = module.types.get(module.funcs.get(function_type_id).ty());
    if !function_type.results().is_empty() {
        return Err(WasmValidationError::InvalidStylusInterface(
            "pay_for_memory_grow function must have no return values".to_string(),
        ));
    }
    if function_type.params().len() != 1 {
        return Err(WasmValidationError::InvalidStylusInterface(
            "pay_for_memory_grow function must have one parameter".to_string(),
        ));
    }
    if function_type.params()[0] != ValType::I32 {
        return Err(WasmValidationError::InvalidStylusInterface(
            "pay_for_memory_grow function must have i32 parameter".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use walrus::{FunctionBuilder, FunctionId, Module, ModuleConfig, ValType};

    use crate::hostio;

    use super::*;

    fn add_valid_wasm_function(module: &mut Module) -> FunctionId {
        // Function type should match the entrypoint function
        let mut noop_func =
            FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);

        let i = module.locals.add(ValType::I32);

        noop_func.func_body().i32_const(0).return_();

        let noop_func = noop_func.finish(vec![i], &mut module.funcs);

        module.exports.add("noop", noop_func);

        noop_func
    }

    fn add_invalid_wasm_function(module: &mut Module) -> FunctionId {
        // Function type should match the entrypoint function
        let mut noop_func =
            FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);

        let i = module.locals.add(ValType::I32);

        noop_func.func_body().return_(); // <---The return is missing in the stack

        let noop_func = noop_func.finish(vec![i], &mut module.funcs);

        module.exports.add("noop", noop_func);

        noop_func
    }

    #[test]
    fn test_validate_stylus_wasm() {
        let (mut module, _, _) = hostio::new_module_with_host();
        let factorial = add_valid_wasm_function(&mut module);
        hostio::add_entrypoint(&mut module, factorial);

        validate_stylus_wasm(&mut module).unwrap();
    }

    #[test]
    fn test_validate_invalid_wasm() {
        let (mut module, _, _) = hostio::new_module_with_host();
        let factorial = add_invalid_wasm_function(&mut module);
        hostio::add_entrypoint(&mut module, factorial);

        let result = validate_stylus_wasm(&mut module);
        assert!(result.is_err());

        let WasmValidationError::InvalidWasm(s) = result.err().unwrap() else {
            panic!("Expected InvalidWasm error");
        };
        assert!(s.contains("expected i32 but nothing on stack"));
    }

    #[test]
    fn test_validate_invalid_pay_for_memory_grow_import() {
        let config = ModuleConfig::new();
        let mut module = Module::with_config(config);

        let memory_id = module.memories.add_local(false, false, 1, None, None);
        module.exports.add("memory", memory_id);

        // We are not adding the pay_for_memory_grow import

        let factorial = add_valid_wasm_function(&mut module);
        hostio::add_entrypoint(&mut module, factorial);

        let result = validate_stylus_wasm(&mut module);
        assert!(result.is_err());

        let WasmValidationError::InvalidStylusInterface(s) = result.err().unwrap() else {
            panic!("Expected InvalidStylusInterface error");
        };
        assert!(s.contains("pay_for_memory_grow import not found"));
    }

    #[test]
    fn test_validate_invalid_pay_for_memory_grow_import_function_type() {
        let config = ModuleConfig::new();
        let mut module = Module::with_config(config);

        let memory_id = module.memories.add_local(false, false, 1, None, None);
        module.exports.add("memory", memory_id);

        let pay_for_memory_grow_type = module.types.add(&[], &[]);
        module.add_import_func("vm_hooks", "pay_for_memory_grow", pay_for_memory_grow_type);

        let factorial = add_valid_wasm_function(&mut module);
        hostio::add_entrypoint(&mut module, factorial);

        let result = validate_stylus_wasm(&mut module);
        assert!(result.is_err());

        let WasmValidationError::InvalidStylusInterface(s) = result.err().unwrap() else {
            panic!("Expected InvalidStylusInterface error");
        };
        assert!(s.contains("pay_for_memory_grow function must have one parameter"));
    }

    #[test]
    fn test_validate_invalid_memory_export() {
        let config = ModuleConfig::new();
        let mut module = Module::with_config(config);

        // We are not adding the memory export

        let pay_for_memory_grow_type = module.types.add(&[ValType::I32], &[]);
        module.add_import_func("vm_hooks", "pay_for_memory_grow", pay_for_memory_grow_type);

        let factorial = add_valid_wasm_function(&mut module);
        hostio::add_entrypoint(&mut module, factorial);

        let result = validate_stylus_wasm(&mut module);
        assert!(result.is_err());

        let WasmValidationError::InvalidStylusInterface(s) = result.err().unwrap() else {
            panic!("Expected InvalidStylusInterface error");
        };
        assert!(s.contains("memory export not found"));
    }

    #[test]
    fn test_validate_invalid_user_entrypoint_export() {
        let (mut module, _, _) = hostio::new_module_with_host();
        add_valid_wasm_function(&mut module);
        // We are not adding the user_entrypoint export

        let result = validate_stylus_wasm(&mut module);
        assert!(result.is_err());

        let WasmValidationError::InvalidStylusInterface(s) = result.err().unwrap() else {
            panic!("Expected InvalidStylusInterface error");
        };
        assert!(s.contains("user_entrypoint export not found"));
    }
}
