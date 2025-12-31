use crate::{
    CompilationContext, hostio::host_functions::tx_origin, runtime::RuntimeFunction,
    translation::intermediate_types::signer::ISigner,
};
use walrus::{FunctionBuilder, FunctionId, Module, ValType, ir::BinaryOp};

pub fn inject_signer(module: &mut Module, compilation_ctx: &CompilationContext) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[], &[]);
    let mut builder = function
        .name(RuntimeFunction::InjectSigner.name().to_owned())
        .func_body();

    let (tx_origin_function, _) = tx_origin(module);
    let signer_pointer = module.locals.add(ValType::I32);

    builder
        .i32_const(ISigner::HEAP_SIZE)
        .call(compilation_ctx.allocator)
        .local_tee(signer_pointer);

    // We add 12 to the pointer returned by the allocator because stylus writes 20
    // bytes, and those bytes need to be at the end.
    builder
        .i32_const(12)
        .binop(BinaryOp::I32Add)
        .call(tx_origin_function)
        .local_get(signer_pointer);

    function.finish(vec![], &mut module.funcs)
}
