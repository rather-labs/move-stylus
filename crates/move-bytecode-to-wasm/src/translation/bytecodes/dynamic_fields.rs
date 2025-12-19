use walrus::{
    GlobalId, InstrSeqBuilder, Module, ValType,
    ir::{BinaryOp, Value},
};

use crate::{
    CompilationContext,
    compilation_context::{
        ModuleId,
        reserved_modules::{
            SF_MODULE_NAME_DYNAMIC_FIELD, SF_MODULE_NAME_DYNAMIC_FIELD_NAMED_ID,
            STYLUS_FRAMEWORK_ADDRESS,
        },
    },
    translation::{
        TranslationError,
        intermediate_types::{IntermediateType, VmHandledStruct},
        table::FunctionId,
    },
};

pub fn add_field_borrow_mut_global_var_instructions(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    builder: &mut InstrSeqBuilder,
    dynamic_fields_global_variables: &mut Vec<(GlobalId, IntermediateType)>,
    function_id: &FunctionId,
) -> Result<(), TranslationError> {
    let global_struct_ptr = module.globals.add_local(
        ValType::I32,
        true,
        false,
        walrus::ConstExpr::Value(Value::I32(-1)),
    );
    let field_value_ref_ptr = module.locals.add(ValType::I32);

    // The borrow_mut functions borrows the value of a `Field`, which is the
    // third field. So, to get the struct pointer we just need to go 8 bytes
    // before it.
    builder
        .local_tee(field_value_ref_ptr)
        .i32_const(8)
        .binop(BinaryOp::I32Sub)
        .global_set(global_struct_ptr);

    let borrow_mut_type_instantiations = function_id
        .type_instantiations
        .as_ref()
        .ok_or(TranslationError::DynamicFieldBorrowMutNoTypeInstantiations)?;

    let field_types_instances =
        if function_id.module_id.module_name.as_str() == SF_MODULE_NAME_DYNAMIC_FIELD_NAMED_ID {
            vec![
                borrow_mut_type_instantiations[1].clone(),
                borrow_mut_type_instantiations[2].clone(),
            ]
        } else {
            vec![
                borrow_mut_type_instantiations[0].clone(),
                borrow_mut_type_instantiations[1].clone(),
            ]
        };

    let dynamic_fields_module_id =
        ModuleId::new(STYLUS_FRAMEWORK_ADDRESS, SF_MODULE_NAME_DYNAMIC_FIELD);

    let dynamic_fields_module = compilation_ctx.get_module_data_by_id(&dynamic_fields_module_id)?;

    let field_struct = dynamic_fields_module.structs.get_by_identifier("Field")?;

    dynamic_fields_global_variables.push((
        global_struct_ptr,
        IntermediateType::IGenericStructInstance {
            module_id: dynamic_fields_module_id,
            index: field_struct.index(),
            types: field_types_instances,
            vm_handled_struct: VmHandledStruct::None,
        },
    ));

    // Leave in the stack the field reference pointer
    builder.local_get(field_value_ref_ptr);

    Ok(())
}
