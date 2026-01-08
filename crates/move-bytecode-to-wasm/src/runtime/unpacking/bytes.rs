use crate::{
    CompilationContext,
    runtime::{RuntimeFunction, RuntimeFunctionError},
};
use walrus::{FunctionBuilder, FunctionId, Module, ValType, ir::BinaryOp};

pub fn unpack_bytes_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function
        .name(RuntimeFunction::UnpackBytes.name().to_owned())
        .func_body();

    let reader_pointer = module.locals.add(ValType::I32);

    // Advance the reader pointer by 32
    builder
        .local_get(reader_pointer)
        .i32_const(32)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.globals.calldata_reader_pointer);

    builder.local_get(reader_pointer);

    Ok(function.finish(vec![reader_pointer], &mut module.funcs))
}
