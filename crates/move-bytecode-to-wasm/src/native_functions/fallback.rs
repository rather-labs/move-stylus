use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{LoadKind, MemArg},
};

use crate::{
    CompilationContext, compilation_context::ModuleId, data::DATA_CALLDATA_OFFSET,
    runtime::RuntimeFunction,
};

use super::NativeFunction;

/// Converts the raw calldata bytes into a vector<u8>
pub fn add_calldata_as_vector_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    module_id: &ModuleId,
) -> FunctionId {
    let bytes_to_vec_fn = RuntimeFunction::BytesToVec
        .get(module, Some(compilation_ctx))
        .expect("BytesToVec runtime function should be available");

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);

    let calldata_ptr = module.locals.add(ValType::I32);

    let mut builder = function
        .name(NativeFunction::get_function_name(
            NativeFunction::NATIVE_CALLDATA_AS_VECTOR,
            module_id,
        ))
        .func_body();

    // Push calldata_ptr (first argument)
    builder.local_get(calldata_ptr);

    // Load and push the calldata length from DATA_CALLDATA_OFFSET (second argument)
    builder.i32_const(DATA_CALLDATA_OFFSET).load(
        compilation_ctx.memory_id,
        LoadKind::I32 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );

    // Call the runtime function to convert bytes to vector
    builder.call(bytes_to_vec_fn);

    function.finish(vec![calldata_ptr], &mut module.funcs)
}

/// Returns the length of the calldata
pub fn add_calldata_length_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    module_id: &ModuleId,
) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let calldata_ptr = module.locals.add(ValType::I32);

    let mut builder = function
        .name(NativeFunction::get_function_name(
            NativeFunction::NATIVE_CALLDATA_LENGTH,
            module_id,
        ))
        .func_body();

    // Load the length of the calldata
    builder.i32_const(DATA_CALLDATA_OFFSET).load(
        compilation_ctx.memory_id,
        LoadKind::I32 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );

    function.finish(vec![calldata_ptr], &mut module.funcs)
}
