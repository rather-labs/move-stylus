use crate::{
    CompilationContext,
    runtime::{RuntimeFunction, RuntimeFunctionError},
};
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{LoadKind, MemArg, StoreKind},
};

pub fn pack_enum_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32], &[]);
    let mut builder = function
        .name(RuntimeFunction::PackEnum.name().to_owned())
        .func_body();

    let enum_ptr = module.locals.add(ValType::I32);
    let writer_pointer = module.locals.add(ValType::I32);

    // Little-endian to Big-endian
    let swap_i32_bytes_function = RuntimeFunction::SwapI32Bytes.get(module, None)?;

    builder.local_get(writer_pointer);

    // Read variant number from enum pointer
    builder
        .local_get(enum_ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .call(swap_i32_bytes_function);

    // Store the variant number at the writer pointer (left-padded to 32 bytes)
    builder.store(
        compilation_ctx.memory_id,
        StoreKind::I32 { atomic: false },
        MemArg {
            align: 0,
            // ABI is left-padded to 32 bytes
            offset: 28,
        },
    );

    Ok(function.finish(vec![enum_ptr, writer_pointer], &mut module.funcs))
}
