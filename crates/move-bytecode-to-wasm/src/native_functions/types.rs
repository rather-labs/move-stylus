use walrus::{FunctionBuilder, FunctionId, Module, ValType};

use crate::{
    CompilationContext,
    compilation_context::ModuleId,
    translation::intermediate_types::{IntermediateType, structs::IStructType},
};

use super::NativeFunction;

/// Checks if the given signature token is a one-time witness type.
//
// OTW (One-time witness) types are structs with the following requirements:
// i. Their name is the upper-case version of the module's name.
// ii. They have no fields (or a single boolean field).
// iii. They have no type parameters.
// iv. They have only the 'drop' ability.
pub fn add_is_one_time_witness_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
    module_id: &ModuleId,
) -> FunctionId {
    // TODO: should we check if itype is a reference to a struct here?
    let name = NativeFunction::get_generic_function_name(
        NativeFunction::NATIVE_IS_ONE_TIME_WITNESS,
        compilation_ctx,
        &[itype],
        module_id,
    );

    if let Some(function) = module.funcs.by_name(&name) {
        return function;
    };

    let struct_ = compilation_ctx
        .get_struct_by_intermediate_type(itype)
        .unwrap();

    // Function declaration
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function.name(name).func_body();

    let ptr = module.locals.add(ValType::I32);

    if struct_.type_ == IStructType::OneTimeWitness {
        builder.i32_const(1);
    } else {
        builder.i32_const(0);
    }

    function.finish(vec![ptr], &mut module.funcs)
}
