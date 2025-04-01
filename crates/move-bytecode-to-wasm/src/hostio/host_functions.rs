use walrus::{FunctionId, ImportId, Module, ValType};

pub fn add_pay_for_memory_grow(module: &mut Module) -> (FunctionId, ImportId) {
    let pay_for_memory_grow_type = module.types.add(&[ValType::I32], &[]);
    module.add_import_func("vm_hooks", "pay_for_memory_grow", pay_for_memory_grow_type)
}

/// Host function to read the arguments to memory
/// Receives a pointer to the memory, and writes the length of the arguments to it
pub fn read_args(module: &mut Module) -> (FunctionId, ImportId) {
    let read_args_type = module.types.add(&[ValType::I32], &[]);
    module.add_import_func("vm_hooks", "read_args", read_args_type)
}

/// Host function to write the result to memory
/// Receives a pointer to the memory and the length of the result
pub fn write_result(module: &mut Module) -> (FunctionId, ImportId) {
    let write_result_type = module.types.add(&[ValType::I32, ValType::I32], &[]);
    module.add_import_func("vm_hooks", "write_result", write_result_type)
}
