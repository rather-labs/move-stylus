use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, MemArg},
};

use crate::{
    CompilationContext,
    abi_types::{event_encoding::move_signature_to_event_signature_hash, packing::Packable},
    compilation_context::ModuleId,
    hostio::host_functions::emit_log,
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

    // Function arguments
    let struct_ptr = module.locals.add(ValType::I32);

    // Locals
    let writer_pointer = module.locals.add(ValType::I32);
    let calldata_reference_pointer = module.locals.add(ValType::I32);
    let packed_data_begin = module.locals.add(ValType::I32);

    let size = if struct_.solidity_abi_encode_is_dynamic(compilation_ctx) {
        32
    } else {
        struct_.solidity_abi_encode_size(compilation_ctx) as i32
    };

    // If the event is not anonymous, we should emit its signature in the first topic
    if !is_anonymous {
        let data = move_signature_to_event_signature_hash(&struct_, compilation_ctx);

        builder
            .i32_const(32)
            .call(compilation_ctx.allocator)
            .local_tee(writer_pointer)
            .local_set(packed_data_begin);

        println!("data event: {data:?}");

        for (index, chunk) in data.chunks_exact(8).enumerate() {
            println!("saving chunk: {chunk:?}");
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

    // Use the allocator to get a pointer to the end of the calldata
    builder
        .i32_const(size)
        .call(compilation_ctx.allocator)
        .local_tee(writer_pointer)
        .local_set(calldata_reference_pointer);

    if is_anonymous {
        builder.local_set(packed_data_begin);
    }

    // ABI pack the struct before emitting the event
    if struct_.solidity_abi_encode_is_dynamic(compilation_ctx) {
        struct_.add_pack_instructions(
            &mut builder,
            module,
            struct_ptr,
            writer_pointer,
            calldata_reference_pointer,
            compilation_ctx,
            Some(calldata_reference_pointer),
        );
    } else {
        struct_.add_pack_instructions(
            &mut builder,
            module,
            struct_ptr,
            writer_pointer,
            calldata_reference_pointer,
            compilation_ctx,
            None,
        );
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

    // Log 0
    builder.i32_const(1 + indexes as i32).call(emit_log_fn);

    function.finish(vec![struct_ptr], &mut module.funcs)
}

pub fn add_emit_log_fn_2(
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
        .unwrap_or_else(|e| {
            panic!("there wsas an error encoding an struct for storage, found {itype:?}.\n{e}")
        });

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

    // Function arguments
    let struct_ptr = module.locals.add(ValType::I32);

    // Locals
    let writer_pointer = module.locals.add(ValType::I32);
    let calldata_reference_pointer = module.locals.add(ValType::I32);
    let packed_data_begin = module.locals.add(ValType::I32);
    let local = module.locals.add(ValType::I32);

    let mut used_topics = 1;

    // If the event is not anonymous, we should emit its signature in the first topic
    if !is_anonymous {
        let data = move_signature_to_event_signature_hash(&struct_, compilation_ctx);

        builder
            .i32_const(32)
            .call(compilation_ctx.allocator)
            .local_set(writer_pointer);

        println!("data event: {data:?}");

        for (index, chunk) in data.chunks_exact(8).enumerate() {
            println!("saving chunk: {chunk:?}");
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

        // Log 0
        builder
            .local_get(writer_pointer)
            .i32_const(32)
            .i32_const(1)
            .call(emit_log_fn);

        used_topics += 1;
    }

    let mut field_offset = 0;
    for field in struct_.fields.iter() {
        // Get the pointer to the field
        builder.local_get(struct_ptr).load(
            compilation_ctx.memory_id,
            walrus::ir::LoadKind::I32 { atomic: false },
            MemArg {
                offset: field_offset,
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
            | IntermediateType::IAddress
            | IntermediateType::IVector(_) => {
                builder
                    .i32_const(32)
                    .call(compilation_ctx.allocator)
                    .local_tee(writer_pointer)
                    .local_tee(calldata_reference_pointer)
                    .local_set(packed_data_begin);

                field.add_pack_instructions(
                    &mut builder,
                    module,
                    local,
                    writer_pointer,
                    calldata_reference_pointer,
                    compilation_ctx,
                );
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

                let size = if struct_.solidity_abi_encode_is_dynamic(compilation_ctx) {
                    32
                } else {
                    struct_.solidity_abi_encode_size(compilation_ctx) as i32
                };
                // Use the allocator to get a pointer to the end of the calldata
                builder
                    .i32_const(size)
                    .call(compilation_ctx.allocator)
                    .local_tee(writer_pointer)
                    .local_tee(calldata_reference_pointer)
                    .local_set(packed_data_begin);

                // ABI pack the struct before emitting the event
                if struct_.solidity_abi_encode_is_dynamic(compilation_ctx) {
                    struct_.add_pack_instructions(
                        &mut builder,
                        module,
                        struct_ptr,
                        writer_pointer,
                        calldata_reference_pointer,
                        compilation_ctx,
                        Some(calldata_reference_pointer),
                    );
                } else {
                    struct_.add_pack_instructions(
                        &mut builder,
                        module,
                        struct_ptr,
                        writer_pointer,
                        calldata_reference_pointer,
                        compilation_ctx,
                        None,
                    );
                }
            }
            IntermediateType::IEnum(_) => todo!(),
            IntermediateType::IRef(intermediate_type) => todo!(),
            IntermediateType::IMutRef(intermediate_type) => todo!(),
            IntermediateType::ITypeParameter(_) => todo!(),
            _ => todo!(),
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

        // If we used all indexed topics, we emit the fields in the LOG4 slot, that is, the data
        // topic
        let topic = if used_topics == 4 || used_topics > indexes {
            0
        } else {
            used_topics
        };
        builder.i32_const(topic as i32).call(emit_log_fn);
        used_topics += 1;
        field_offset += 4;
    }

    function.finish(vec![struct_ptr], &mut module.funcs)
}

/*
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
        .unwrap_or_else(|e| {
            panic!("there wsas an error encoding an struct for storage, found {itype:?}.\n{e}")
        });

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

    // Function arguments
    let struct_ptr = module.locals.add(ValType::I32);

    // Locals
    let writer_pointer = module.locals.add(ValType::I32);
    let calldata_reference_pointer = module.locals.add(ValType::I32);
    let packed_data_begin = module.locals.add(ValType::I32);
    let local = module.locals.add(ValType::I32);

    let mut used_topics = 1;

    // If the event is not anonymous, we should emit its signature in the first topic
    if !is_anonymous {
        let data = move_signature_to_event_signature_hash(&struct_, compilation_ctx);

        builder
            .i32_const(32)
            .call(compilation_ctx.allocator)
            .local_set(writer_pointer);

        println!("data event: {data:?}");

        for (index, chunk) in data.chunks_exact(8).enumerate() {
            println!("saving chunk: {chunk:?}");
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

        // Log 0
        builder
            .local_get(writer_pointer)
            .i32_const(32)
            .i32_const(1)
            .call(emit_log_fn);

        used_topics += 1;
    }

    let mut field_offset = 0;
    for field in struct_.fields.iter() {
        // Get the pointer to the field
        builder.local_get(struct_ptr).load(
            compilation_ctx.memory_id,
            walrus::ir::LoadKind::I32 { atomic: false },
            MemArg {
                offset: field_offset,
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
            | IntermediateType::IAddress
            | IntermediateType::IVector(_) => {
                builder
                    .i32_const(32)
                    .call(compilation_ctx.allocator)
                    .local_tee(writer_pointer)
                    .local_tee(calldata_reference_pointer)
                    .local_set(packed_data_begin);

                field.add_pack_instructions(
                    &mut builder,
                    module,
                    local,
                    writer_pointer,
                    calldata_reference_pointer,
                    compilation_ctx,
                );
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

                let size = if struct_.solidity_abi_encode_is_dynamic(compilation_ctx) {
                    32
                } else {
                    struct_.solidity_abi_encode_size(compilation_ctx) as i32
                };
                // Use the allocator to get a pointer to the end of the calldata
                builder
                    .i32_const(size)
                    .call(compilation_ctx.allocator)
                    .local_tee(writer_pointer)
                    .local_tee(calldata_reference_pointer)
                    .local_set(packed_data_begin);

                // ABI pack the struct before emitting the event
                if struct_.solidity_abi_encode_is_dynamic(compilation_ctx) {
                    struct_.add_pack_instructions(
                        &mut builder,
                        module,
                        struct_ptr,
                        writer_pointer,
                        calldata_reference_pointer,
                        compilation_ctx,
                        Some(calldata_reference_pointer),
                    );
                } else {
                    struct_.add_pack_instructions(
                        &mut builder,
                        module,
                        struct_ptr,
                        writer_pointer,
                        calldata_reference_pointer,
                        compilation_ctx,
                        None,
                    );
                }
            }
            IntermediateType::IEnum(_) => todo!(),
            IntermediateType::IRef(intermediate_type) => todo!(),
            IntermediateType::IMutRef(intermediate_type) => todo!(),
            IntermediateType::ITypeParameter(_) => todo!(),
            _ => todo!(),
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

        // If we used all indexed topics, we emit the fields in the LOG4 slot, that is, the data
        // topic
        let topic = if used_topics == 4 || used_topics > indexes {
            0
        } else {
            used_topics
        };
        builder.i32_const(topic as i32).call(emit_log_fn);
        used_topics += 1;
        field_offset += 4;
    }

    function.finish(vec![struct_ptr], &mut module.funcs)
}

*/
