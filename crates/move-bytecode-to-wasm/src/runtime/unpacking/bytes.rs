use crate::{
    CompilationContext,
    runtime::{RuntimeFunction, RuntimeFunctionError},
};
use walrus::{FunctionBuilder, FunctionId, Module, ValType, ir::BinaryOp};

pub fn unpack_bytes_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    let mut function_builder =
        FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut function_body = function_builder.func_body();

    let reader_pointer = module.locals.add(ValType::I32);

    // Advance the reader pointer by 32
    function_body
        .local_get(reader_pointer)
        .i32_const(32)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.calldata_reader_pointer);

    function_body.local_get(reader_pointer);

    function_builder.name(RuntimeFunction::UnpackBytes.name().to_owned());
    Ok(function_builder.finish(vec![reader_pointer], &mut module.funcs))
}
