use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, MemArg},
};

use crate::{
    CompilationContext,
    abi_types::{event_encoding::move_signature_to_event_signature_hash, packing::Packable},
    compilation_context::ModuleId,
    hostio::host_functions::{emit_log, native_keccak256},
    translation::intermediate_types::{IntermediateType, structs::IStructType},
};

use super::NativeFunction;

pub fn add_emit_log_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
    module_id: &ModuleId,
) -> FunctionId {
    let name =
        NativeFunction::get_generic_function_name(NativeFunction::NATIVE_EMIT, &[itype], module_id);
    if let Some(function) = module.funcs.by_name(&name) {
        return function;
    };

    let struct_ = compilation_ctx
        .get_struct_by_intermediate_type(itype)
        .unwrap();

    // TODO: This should be a compile error not a panic
    let IStructType::Event {
        indexes,
        is_anonymous,
    } = struct_.type_
    else {
        panic!(
            "trying to emit log with the struct {} which is not an event",
            struct_.identifier
        );
    };

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
    let mut builder = function.name(name).func_body();

    let (emit_log_fn, _) = emit_log(module);
    let (native_keccak, _) = native_keccak256(module);

    // Function arguments
    let struct_ptr = module.locals.add(ValType::I32);

    // Locals
    let writer_pointer = module.locals.add(ValType::I32);
    let calldata_reference_pointer = module.locals.add(ValType::I32);
    let packed_data_begin = module.locals.add(ValType::I32);
    let local = module.locals.add(ValType::I32);
    let abi_encoded_data_length = module.locals.add(ValType::I32);

    let (print_i32, _, _, _, _, _) = crate::declare_host_debug_functions!(module);

    // Before encoding the event, abi encode complex fields such as structs, vectors and strings,
    // then, if those fields are dynamic, we just put the keccak256 in the corresponding topic,
    // otherwise, we copy the whole encoding
    let mut event_fields_encoded_data = Vec::new();

    // ABI pack the struct before emitting the event
    for (field_index, field) in struct_.fields.iter().enumerate() {
        // Get the pointer to the field
        builder
            .local_get(struct_ptr)
            .load(
                compilation_ctx.memory_id,
                walrus::ir::LoadKind::I32 { atomic: false },
                MemArg {
                    offset: field_index as u32 * 4,
                    align: 0,
                },
            )
            .local_set(local);

        match field {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64
            | IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::IAddress => {
                event_fields_encoded_data.push(None);
                continue;
            }
            IntermediateType::IVector(_) => {
                // Get the pointer to the field
                builder
                    .local_get(struct_ptr)
                    .load(
                        compilation_ctx.memory_id,
                        walrus::ir::LoadKind::I32 { atomic: false },
                        MemArg {
                            offset: field_index as u32 * 4,
                            align: 0,
                        },
                    )
                    .local_set(local);

                let abi_encoded_data_writer_pointer = module.locals.add(ValType::I32);
                let abi_encoded_data_calldata_reference_pointer = module.locals.add(ValType::I32);

                builder
                    .i32_const(32)
                    .call(compilation_ctx.allocator)
                    .local_tee(abi_encoded_data_writer_pointer)
                    .local_set(abi_encoded_data_calldata_reference_pointer);

                field.add_pack_instructions(
                    &mut builder,
                    module,
                    local,
                    abi_encoded_data_writer_pointer,
                    abi_encoded_data_calldata_reference_pointer,
                    compilation_ctx,
                );

                builder
                    .i32_const(0)
                    .call(compilation_ctx.allocator)
                    .local_set(abi_encoded_data_writer_pointer);

                event_fields_encoded_data.push(Some((
                    abi_encoded_data_calldata_reference_pointer,
                    abi_encoded_data_writer_pointer,
                )));
            }
            IntermediateType::IStruct {
                module_id, index, ..
            }
            | IntermediateType::IGenericStructInstance {
                module_id, index, ..
            } => {
                let struct_ = compilation_ctx
                    .get_struct_by_index(module_id, *index)
                    .unwrap();

                let struct_ = if let IntermediateType::IGenericStructInstance { types, .. } = field
                {
                    &struct_.instantiate(types)
                } else {
                    struct_
                };

                let abi_encoded_data_writer_pointer = module.locals.add(ValType::I32);
                let abi_encoded_data_calldata_reference_pointer = module.locals.add(ValType::I32);

                let size = if struct_.solidity_abi_encode_is_dynamic(compilation_ctx) {
                    32
                } else {
                    struct_.solidity_abi_encode_size(compilation_ctx) as i32
                };
                // Use the allocator to get a pointer to the end of the calldata
                builder
                    .i32_const(size)
                    .call(compilation_ctx.allocator)
                    .local_tee(abi_encoded_data_writer_pointer)
                    .local_set(abi_encoded_data_calldata_reference_pointer);

                // ABI pack the struct before emitting the event
                if struct_.solidity_abi_encode_is_dynamic(compilation_ctx) {
                    struct_.add_pack_instructions(
                        &mut builder,
                        module,
                        struct_ptr,
                        abi_encoded_data_writer_pointer,
                        abi_encoded_data_calldata_reference_pointer,
                        compilation_ctx,
                        Some(abi_encoded_data_calldata_reference_pointer),
                    );
                } else {
                    struct_.add_pack_instructions(
                        &mut builder,
                        module,
                        struct_ptr,
                        abi_encoded_data_writer_pointer,
                        abi_encoded_data_calldata_reference_pointer,
                        compilation_ctx,
                        None,
                    );
                }

                builder
                    .i32_const(0)
                    .call(compilation_ctx.allocator)
                    .local_set(abi_encoded_data_writer_pointer);

                event_fields_encoded_data.push(Some((
                    abi_encoded_data_calldata_reference_pointer,
                    abi_encoded_data_writer_pointer,
                )));
            }
            IntermediateType::IEnum(_) => todo!(),
            _ => panic!("invalid event field {field:?}"),
        }
    }

    // If the event is not anonymous, we should emit its signature in the first topic
    if !is_anonymous {
        let data = move_signature_to_event_signature_hash(&struct_, compilation_ctx);

        builder
            .i32_const(32)
            .call(compilation_ctx.allocator)
            .local_tee(writer_pointer)
            .local_set(packed_data_begin);

        for (index, chunk) in data.chunks_exact(8).enumerate() {
            builder
                .local_get(writer_pointer)
                .i64_const(i64::from_le_bytes(chunk.try_into().unwrap()))
                .store(
                    compilation_ctx.memory_id,
                    walrus::ir::StoreKind::I64 { atomic: false },
                    MemArg {
                        offset: index as u32 * 8,
                        align: 0,
                    },
                );
        }
    }

    if is_anonymous {
        builder.local_set(packed_data_begin);
    }

    // ABI pack the struct before emitting the event
    for ((field_index, field), abi_encoded_data) in struct_
        .fields
        .iter()
        .enumerate()
        .zip(event_fields_encoded_data)
    {
        let is_indexed = field_index < indexes as usize;

        // Get the pointer to the field
        builder.local_get(struct_ptr).load(
            compilation_ctx.memory_id,
            walrus::ir::LoadKind::I32 { atomic: false },
            MemArg {
                offset: field_index as u32 * 4,
                align: 0,
            },
        );
        // If it is a stack type, we need to perform another load
        // TODO: u64 case
        if field.is_stack_type() {
            builder
                .load(
                    compilation_ctx.memory_id,
                    walrus::ir::LoadKind::I32 { atomic: false },
                    MemArg {
                        offset: 0,
                        align: 0,
                    },
                )
                .local_set(local);
        } else {
            builder.local_set(local);
        }

        match field {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64
            | IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::IAddress => {
                builder
                    .i32_const(32)
                    .call(compilation_ctx.allocator)
                    .local_tee(writer_pointer)
                    .local_set(calldata_reference_pointer);

                field.add_pack_instructions(
                    &mut builder,
                    module,
                    local,
                    writer_pointer,
                    calldata_reference_pointer,
                    compilation_ctx,
                );

                // Add 32 to the writer pointer to write the next topic
                builder
                    .i32_const(32)
                    .local_get(writer_pointer)
                    .binop(BinaryOp::I32Add)
                    .local_tee(writer_pointer)
                    .local_set(calldata_reference_pointer);
            }
            IntermediateType::IVector(_)
            | IntermediateType::IStruct { .. }
            | IntermediateType::IGenericStructInstance { .. } => {
                let Some((encode_start, encode_end)) = abi_encoded_data else {
                    panic!(
                        "there was an error instantiating an emit event function: vector does not have abi encoded data"
                    )
                };

                builder
                    .local_get(encode_end)
                    .local_get(encode_start)
                    .binop(BinaryOp::I32Sub)
                    .local_set(abi_encoded_data_length);

                builder.local_get(encode_start).call(print_i32);
                builder.local_get(encode_end).call(print_i32);

                builder
                    .local_get(abi_encoded_data_length)
                    .call(compilation_ctx.allocator)
                    .drop();

                // If the vector is indexed, we need to calculate the keccak256 of its values and
                // store them in the topic
                if is_indexed {
                    builder
                        .local_get(encode_start)
                        .local_get(abi_encoded_data_length)
                        .local_get(writer_pointer)
                        .call(native_keccak);

                    builder
                        .i32_const(32)
                        .local_get(writer_pointer)
                        .binop(BinaryOp::I32Add)
                        .local_tee(writer_pointer)
                        .local_set(calldata_reference_pointer);
                } else {
                    // Copy the abi encoded data to its place in the event data
                    builder
                        .local_get(writer_pointer)
                        .local_get(encode_start)
                        .local_get(abi_encoded_data_length)
                        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

                    // Update the calldata reference to the start of the new field (vector encoding
                    // moves the writer pointer to the end of the encoding)
                    builder
                        .local_get(writer_pointer)
                        .local_get(abi_encoded_data_length)
                        .binop(BinaryOp::I32Add)
                        .local_tee(writer_pointer)
                        .local_set(calldata_reference_pointer);
                }
            }
            IntermediateType::IEnum(_) => todo!(),
            _ => panic!("invalid event field {field:?}"),
        }
    }

    // Emit the event with the ABI packed struct

    // Beginning of the packed data
    builder.local_get(packed_data_begin);

    // Use the allocator to get a pointer to the end of the calldata
    builder
        .i32_const(0)
        .call(compilation_ctx.allocator)
        .local_get(packed_data_begin)
        .binop(BinaryOp::I32Sub);

    let indexes = if is_anonymous { indexes } else { 1 + indexes } as i32;
    builder.i32_const(indexes).call(emit_log_fn);

    function.finish(vec![struct_ptr], &mut module.funcs)
}
