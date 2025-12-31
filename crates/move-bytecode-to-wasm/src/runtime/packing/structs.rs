use crate::{
    CompilationContext,
    abi_types::packing::Packable,
    runtime::{RuntimeFunction, RuntimeFunctionError},
    translation::intermediate_types::{
        IntermediateType,
        structs::{IStruct, IStructType},
    },
};
use std::collections::HashMap;
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg},
};

pub fn pack_struct_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let name = RuntimeFunction::PackStruct.get_generic_function_name(compilation_ctx, &[itype])?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    }

    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32, ValType::I32],
        &[],
    );

    let mut builder: walrus::InstrSeqBuilder<'_> = function.name(name).func_body();

    // For event structs, create a new struct with only data fields (excluding indexed fields)
    let struct_ = {
        let struct_ = compilation_ctx.get_struct_by_intermediate_type(itype)?;
        if let IStructType::Event { indexes, .. } = struct_.type_ {
            IStruct::new(
                move_binary_format::file_format::StructDefinitionIndex(0),
                &format!("{}Data", struct_.identifier),
                struct_.fields[indexes as usize..]
                    .iter()
                    .map(|t| (None, t.clone()))
                    .collect(),
                HashMap::new(),
                false,
                IStructType::Common,
            )
        } else {
            struct_.into_owned()
        }
    };

    // Arguments
    let struct_pointer = module.locals.add(ValType::I32);
    let writer_pointer = module.locals.add(ValType::I32);
    let calldata_reference_pointer = module.locals.add(ValType::I32);
    let is_nested = module.locals.add(ValType::I32);

    let val_32 = module.locals.add(ValType::I32);
    let val_64 = module.locals.add(ValType::I64);
    let reference_value = module.locals.add(ValType::I32);

    let data_ptr = module.locals.add(ValType::I32);
    let inner_data_reference = module.locals.add(ValType::I32);

    // Compute the size before the closure since closures that return () cannot use ?
    let struct_size = struct_.solidity_abi_encode_size(compilation_ctx)? as i32;
    let pack_u32_function = RuntimeFunction::PackU32.get(module, Some(compilation_ctx))?;

    // If is_nested is 1, means we are packing an struct inside a struct and that the struct is dynamic.
    builder.local_get(is_nested).if_else(
        None,
        |then| {
            // Allocate memory for the packed value. Set the data_ptr the beginning, since
            // we are going to pack the values from there
            then.i32_const(struct_size)
                .call(compilation_ctx.allocator)
                .local_tee(data_ptr)
                .local_tee(inner_data_reference);

            // The pointer in the packed data must be relative to the calldata_reference_pointer,
            // so we substract calldata_reference_pointer from the writer_pointer
            then.local_get(calldata_reference_pointer)
                .binop(BinaryOp::I32Sub)
                .local_set(reference_value);

            // The result is saved where calldata_reference_pointer is pointing at, the value will
            // be the address where the struct  values are packed, using as origin
            // calldata_reference_pointer
            then.local_get(reference_value)
                .local_get(writer_pointer)
                .call(pack_u32_function);
        },
        |else_| {
            else_.local_get(writer_pointer).local_set(data_ptr);
        },
    );

    // Load the value to be written in the calldata, if it is a stack value we need to double
    // reference a pointer, otherwise we read the pointer and leave the stack value in the
    // stack
    for (index, field) in struct_.fields.iter().enumerate() {
        // Load field's intermediate pointer
        builder.local_get(struct_pointer).load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: index as u32 * 4,
            },
        );

        // Load the value
        let field_local = match field {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64 => {
                let val = match ValType::try_from(field)? {
                    ValType::I64 => val_64,
                    _ => val_32,
                };

                builder
                    .load(
                        compilation_ctx.memory_id,
                        field.load_kind()?,
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_set(val);

                val
            }
            _ => {
                builder.local_set(val_32);
                val_32
            }
        };

        // If is_nested == 0, means we are not packing this struct
        // dynamically, so, we can set inner_data_reference as the root reference pointer
        builder.block(None, |block| {
            let block_id = block.id();
            block.local_get(is_nested).br_if(block_id);

            block
                .local_get(calldata_reference_pointer)
                .local_set(inner_data_reference);
        });

        // If the field to pack is a struct, it will be packed dynamically, that means, in the
        // current offset of writer pointer, we are going to write the offset where we can find
        // the struct
        let advancement: Result<usize, RuntimeFunctionError> = match field {
            IntermediateType::IStruct { .. } | IntermediateType::IGenericStructInstance { .. } => {
                let child_struct = compilation_ctx.get_struct_by_intermediate_type(field)?;

                if child_struct.solidity_abi_encode_is_dynamic(compilation_ctx)? {
                    field.add_pack_instructions_dynamic(
                        &mut builder,
                        module,
                        field_local,
                        data_ptr,
                        inner_data_reference,
                        compilation_ctx,
                    )?;
                    Ok(32)
                } else {
                    field.add_pack_instructions(
                        &mut builder,
                        module,
                        field_local,
                        data_ptr,
                        inner_data_reference,
                        compilation_ctx,
                    )?;
                    Ok(field.encoded_size(compilation_ctx)?)
                }
            }
            _ => {
                field.add_pack_instructions(
                    &mut builder,
                    module,
                    field_local,
                    data_ptr,
                    inner_data_reference,
                    compilation_ctx,
                )?;
                Ok(32)
            }
        };

        // The value of advacement depends on the following conditions:
        // - If the field we are encoding is a static struct, the pointer must be advanced the size
        //   of the tuple that represents the struct.
        // - If the field we are encoding is a dynamic struct, we just need to advance the pointer
        //   32 bytes because in the argument's place there is only a pointer to where the
        //   struct's values are packed
        // - If it is not a struct:
        //   - If it is a static field it will occupy 32 bytes,
        //   - if it is a dynamic field, the offset pointing to where to find the values will be
        //     written, also occuping 32 bytes.
        let advancement = advancement?;
        builder
            .i32_const(advancement as i32)
            .local_get(data_ptr)
            .binop(BinaryOp::I32Add)
            .local_set(data_ptr);
    }

    Ok(function.finish(
        vec![
            struct_pointer,
            writer_pointer,
            calldata_reference_pointer,
            is_nested,
        ],
        &mut module.funcs,
    ))
}
