use std::{collections::HashMap, sync::Arc};

use move_binary_format::file_format::FieldHandleIndex;
use walrus::{
    FunctionBuilder, FunctionId, InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, ExtendedLoad, LoadKind, MemArg, StoreKind},
};

use crate::{
    CompilationContext,
    abi_types::{event_encoding::move_signature_to_event_signature_hash, packing::Packable},
    compilation_context::ModuleId,
    data::RuntimeErrorData,
    hostio::host_functions::{emit_log, native_keccak256},
    runtime::RuntimeFunction,
    translation::intermediate_types::{
        IntermediateType,
        structs::{IStruct, IStructType},
    },
    vm_handled_types::{VmHandledType, string::String_},
    wasm_builder_extensions::WasmBuilderExtension,
};

use super::{NativeFunction, error::NativeFunctionError};

/// This function ABI-encodes an event struct following the Solidity language specification:
///
/// https://docs.soliditylang.org/en/latest/abi-spec.html#events
///
/// Dynamic structures are first ABI-encoded in memory. Then:
/// * If a dynamic structure is part of a topic, its Keccak-256 hash is computed over the encoded
///   memory region and placed in the corresponding topic.
/// * Otherwise, if it is part of the event data, the encoded memory is copied into the data section
///   of the event
pub fn add_emit_log_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
    itype: &IntermediateType,
    module_id: &ModuleId,
) -> Result<FunctionId, NativeFunctionError> {
    let name = NativeFunction::get_generic_function_name(
        NativeFunction::NATIVE_EMIT,
        compilation_ctx,
        &[itype],
        module_id,
    )?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    };

    let struct_ = compilation_ctx.get_struct_by_intermediate_type(itype)?;

    let IStructType::Event {
        indexes,
        is_anonymous,
    } = struct_.type_
    else {
        return Err(NativeFunctionError::EmitFunctionNoEvent(struct_.identifier));
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
    let local_64 = module.locals.add(ValType::I64);
    let abi_encoded_data_length = module.locals.add(ValType::I32);

    // Before encoding the event, abi encode complex fields such as structs, vectors and strings,
    // then, if those fields are dynamic, we just put the keccak256 in the corresponding topic,
    // otherwise, we copy the whole encoding
    let mut event_fields_encoded_data = Vec::new();

    for (field_index, field) in struct_.fields[..indexes as usize].iter().enumerate() {
        match field {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64
            | IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::IAddress
            | IntermediateType::IEnum { .. } => {
                event_fields_encoded_data.push(None);
                continue;
            }
            IntermediateType::IVector(inner) => {
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

                let data_begin = module.locals.add(ValType::I32);

                builder
                    .get_memory_curret_position(compilation_ctx)
                    .local_set(data_begin);

                let data_end = module.locals.add(ValType::I32);

                add_encode_indexed_vector_instructions(
                    module,
                    &mut builder,
                    compilation_ctx,
                    runtime_error_data,
                    inner,
                    local,
                )?;

                builder
                    .get_memory_curret_position(compilation_ctx)
                    .local_set(data_end);

                event_fields_encoded_data.push(Some((data_begin, data_end)));
            }
            IntermediateType::IStruct {
                module_id,
                index: struct_index,
                ..
            } if String_::is_vm_type(module_id, *struct_index, compilation_ctx)? => {
                let value = module.locals.add(ValType::I32);
                builder
                    .local_get(struct_ptr)
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            offset: field_index as u32 * 4,
                            align: 0,
                        },
                    )
                    .local_set(value);

                let data_begin = module.locals.add(ValType::I32);

                builder
                    .get_memory_curret_position(compilation_ctx)
                    .local_set(data_begin);

                add_encode_indexed_string(module, &mut builder, compilation_ctx, value, false)?;

                let data_end = module.locals.add(ValType::I32);
                builder
                    .get_memory_curret_position(compilation_ctx)
                    .local_set(data_end);

                event_fields_encoded_data.push(Some((data_begin, data_end)))
            }
            IntermediateType::IStruct {
                module_id, index, ..
            }
            | IntermediateType::IGenericStructInstance {
                module_id, index, ..
            } => {
                let struct_ = compilation_ctx.get_struct_by_index(module_id, *index)?;

                let struct_ = if let IntermediateType::IGenericStructInstance { types, .. } = field
                {
                    &struct_.instantiate(types)
                } else {
                    struct_
                };

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

                let data_begin = module.locals.add(ValType::I32);

                builder
                    .get_memory_curret_position(compilation_ctx)
                    .local_set(data_begin);

                add_encode_indexed_struct_instructions(
                    module,
                    &mut builder,
                    compilation_ctx,
                    runtime_error_data,
                    struct_,
                    local,
                )?;

                let data_end = module.locals.add(ValType::I32);
                builder
                    .get_memory_curret_position(compilation_ctx)
                    .local_set(data_end);

                event_fields_encoded_data.push(Some((data_begin, data_end)));
            }
            _ => {
                return Err(NativeFunctionError::EmitFunctionInvalidEventField(
                    field.clone(),
                ));
            }
        }
    }

    // If the event is not anonymous, we should emit its signature in the first topic
    if !is_anonymous {
        let data = move_signature_to_event_signature_hash(&struct_, compilation_ctx)?;

        builder
            .i32_const(32)
            .call(compilation_ctx.allocator)
            .local_tee(writer_pointer)
            .local_set(packed_data_begin);

        for (index, chunk) in data.chunks_exact(8).enumerate() {
            builder
                .local_get(writer_pointer)
                .i64_const(i64::from_le_bytes(
                    chunk
                        .try_into()
                        .map_err(|_| NativeFunctionError::I64InvalidArraySize)?,
                ))
                .store(
                    compilation_ctx.memory_id,
                    walrus::ir::StoreKind::I64 { atomic: false },
                    MemArg {
                        offset: index as u32 * 8,
                        align: 0,
                    },
                );
        }
    } else {
        builder
            .i32_const(0)
            .call(compilation_ctx.allocator)
            .local_set(packed_data_begin);
    }

    // ABI pack the struct before emitting the event
    // First process the indexed ones
    for ((field_index, field), abi_encoded_data) in struct_.fields[..indexes as usize]
        .iter()
        .enumerate()
        .zip(event_fields_encoded_data)
    {
        // Get the pointer to the field
        builder.local_get(struct_ptr).load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                offset: field_index as u32 * 4,
                align: 0,
            },
        );

        // If it is a stack type, we need to perform another load
        let local = if field.is_stack_type()? {
            let local = match ValType::try_from(field)? {
                ValType::I64 => local_64,
                _ => local,
            };

            builder
                .load(
                    compilation_ctx.memory_id,
                    field.load_kind()?,
                    MemArg {
                        offset: 0,
                        align: 0,
                    },
                )
                .local_set(local);

            local
        } else {
            builder.local_set(local);
            local
        };

        match field {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64
            | IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::IEnum { .. }
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
                    Some(runtime_error_data),
                    None,
                )?;
            }
            IntermediateType::IVector(_)
            | IntermediateType::IStruct { .. }
            | IntermediateType::IGenericStructInstance { .. } => {
                let Some((encode_start, encode_end)) = abi_encoded_data else {
                    return Err(NativeFunctionError::EmitFunctionInvalidVectorData);
                };
                builder
                    .local_get(encode_end)
                    .local_get(encode_start)
                    .binop(BinaryOp::I32Sub)
                    .local_set(abi_encoded_data_length);

                // If the vector is indexed, we need to calculate the keccak256 of its values and
                // store them in the topic
                builder
                    .i32_const(32)
                    .call(compilation_ctx.allocator)
                    .local_set(writer_pointer);

                builder
                    .local_get(encode_start)
                    .local_get(abi_encoded_data_length)
                    .local_get(writer_pointer)
                    .call(native_keccak);
            }
            _ => {
                return Err(NativeFunctionError::EmitFunctionInvalidEventField(
                    field.clone(),
                ));
            }
        }
    }

    let data_fields: &[IntermediateType] = &struct_.fields[indexes as usize..];
    let packed_data_length = module.locals.add(ValType::I32);

    if !data_fields.is_empty() {
        // For the data left, we need to generate a "fake" IStruct with the fields that are not
        // included in the topics.
        let fields: Vec<(Option<FieldHandleIndex>, IntermediateType)> =
            data_fields.iter().map(|t| (None, t.clone())).collect();

        let data_struct = IStruct::new(
            move_binary_format::file_format::StructDefinitionIndex(0),
            &format!("{}Data", struct_.identifier),
            fields,
            HashMap::new(),
            false,
            IStructType::Common,
        );

        let abi_encoded_data_calldata_reference_pointer = module.locals.add(ValType::I32);
        let data_struct_ptr = module.locals.add(ValType::I32);

        builder
            .local_get(struct_ptr)
            .i32_const(4 * indexes as i32)
            .binop(BinaryOp::I32Add)
            .local_set(data_struct_ptr);

        let is_dynamic = data_struct.solidity_abi_encode_is_dynamic(compilation_ctx)?;
        let size = if is_dynamic {
            32
        } else {
            data_struct.solidity_abi_encode_size(compilation_ctx)? as i32
        };

        builder
            .i32_const(size)
            .call(compilation_ctx.allocator)
            .local_tee(writer_pointer)
            .local_set(abi_encoded_data_calldata_reference_pointer);

        let pack_struct_function = RuntimeFunction::PackStruct.get_generic(
            module,
            compilation_ctx,
            Some(runtime_error_data),
            &[itype],
        )?;
        if data_struct.solidity_abi_encode_is_dynamic(compilation_ctx)? {
            builder
                .local_get(data_struct_ptr)
                .local_get(writer_pointer)
                .local_get(abi_encoded_data_calldata_reference_pointer)
                .i32_const(1) // is_nested = true
                .call(pack_struct_function);

            // Move 32 bytes back the encoded

            // Destination
            builder.local_get(abi_encoded_data_calldata_reference_pointer);

            // Source
            builder
                .local_get(abi_encoded_data_calldata_reference_pointer)
                .i32_const(32)
                .binop(BinaryOp::I32Add);

            builder
                .i32_const(0)
                .call(compilation_ctx.allocator)
                .local_get(packed_data_begin)
                .i32_const(32)
                .binop(BinaryOp::I32Add)
                .binop(BinaryOp::I32Sub)
                .local_tee(packed_data_length);

            builder.memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);
        } else {
            builder
                .local_get(data_struct_ptr)
                .local_get(writer_pointer)
                .local_get(abi_encoded_data_calldata_reference_pointer)
                .i32_const(0) // is_nested = false
                .call(pack_struct_function);

            builder
                .i32_const(0)
                .call(compilation_ctx.allocator)
                .local_get(packed_data_begin)
                .binop(BinaryOp::I32Sub)
                .local_set(packed_data_length);
        }
    } else {
        builder
            .i32_const(0)
            .call(compilation_ctx.allocator)
            .local_get(packed_data_begin)
            .binop(BinaryOp::I32Sub)
            .local_set(packed_data_length);
    }

    // Beginning of the packed data
    builder
        .local_get(packed_data_begin)
        .local_get(packed_data_length)
        .i32_const(if is_anonymous { indexes } else { 1 + indexes } as i32)
        .call(emit_log_fn);

    Ok(function.finish(vec![struct_ptr], &mut module.funcs))
}

/// Encodes a vector following the ABI spec for events
///
/// https://docs.soliditylang.org/en/latest/abi-spec.html#indexed-event-encoding
fn add_encode_indexed_vector_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
    inner: &IntermediateType,
    vector_ptr: LocalId,
) -> Result<(), NativeFunctionError> {
    // Get the len
    let len = IntermediateType::IU32.add_load_memory_to_local_instructions(
        module,
        builder,
        vector_ptr,
        compilation_ctx.memory_id,
    )?;

    // Skip vector header
    builder.skip_vec_header(vector_ptr).local_set(vector_ptr);

    let i = module.locals.add(ValType::I32);
    builder.i32_const(0).local_set(i);

    let mut inner_result: Result<(), NativeFunctionError> = Ok(());
    match inner {
        // If the data is "simple" we just concatenate things contigously
        IntermediateType::IBool
        | IntermediateType::IU8
        | IntermediateType::IU16
        | IntermediateType::IU32
        | IntermediateType::IU64
        | IntermediateType::IU128
        | IntermediateType::IU256
        | IntermediateType::IAddress => {
            let writer_pointer = module.locals.add(ValType::I32);
            let value = module.locals.add(ValType::try_from(inner)?);
            builder.loop_(None, |loop_| {
                inner_result = (|| {
                    loop_
                        .local_get(vector_ptr)
                        .load(
                            compilation_ctx.memory_id,
                            inner.load_kind()?,
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        )
                        .local_set(value);

                    let loop_id = loop_.id();

                    // Allocate 32 bytes for the encoded data
                    loop_
                        .i32_const(32)
                        .call(compilation_ctx.allocator)
                        .local_set(writer_pointer);

                    inner.add_pack_instructions(
                        loop_,
                        module,
                        value,
                        writer_pointer,
                        writer_pointer,
                        compilation_ctx,
                        Some(runtime_error_data),
                        None,
                    )?;

                    loop_
                        .local_get(vector_ptr)
                        .i32_const(inner.wasm_memory_data_size()?)
                        .binop(BinaryOp::I32Add)
                        .local_set(vector_ptr);

                    // increment i
                    loop_
                        .local_get(i)
                        .i32_const(1)
                        .binop(BinaryOp::I32Add)
                        .local_tee(i);

                    loop_.local_get(len).binop(BinaryOp::I32LtU).br_if(loop_id);

                    Ok(())
                })();
            });
        }
        IntermediateType::IVector(nested_inner) => {
            let value = module.locals.add(ValType::I32);
            builder.i32_const(0).local_set(i);

            builder.loop_(None, |loop_| {
                let loop_id = loop_.id();

                inner_result = (|| {
                    loop_
                        .local_get(vector_ptr)
                        .load(
                            compilation_ctx.memory_id,
                            inner.load_kind()?,
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        )
                        .local_set(value);

                    add_encode_indexed_vector_instructions(
                        module,
                        loop_,
                        compilation_ctx,
                        runtime_error_data,
                        nested_inner,
                        value,
                    )?;

                    loop_
                        .local_get(vector_ptr)
                        .i32_const(inner.wasm_memory_data_size()?)
                        .binop(BinaryOp::I32Add)
                        .local_set(vector_ptr);

                    // increment i
                    loop_
                        .local_get(i)
                        .i32_const(1)
                        .binop(BinaryOp::I32Add)
                        .local_tee(i);

                    loop_.local_get(len).binop(BinaryOp::I32LtU).br_if(loop_id);

                    Ok(())
                })();
            });
        }
        IntermediateType::IStruct {
            module_id,
            index: struct_index,
            ..
        } if String_::is_vm_type(module_id, *struct_index, compilation_ctx)? => {
            let i = module.locals.add(ValType::I32);
            let value = module.locals.add(ValType::I32);
            builder.i32_const(0).local_set(i);

            builder.loop_(None, |loop_| {
                let loop_id = loop_.id();

                inner_result = (|| {
                    loop_
                        .local_get(vector_ptr)
                        .load(
                            compilation_ctx.memory_id,
                            LoadKind::I32 { atomic: false },
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        )
                        .local_set(value);

                    add_encode_indexed_string(module, loop_, compilation_ctx, value, true)?;

                    loop_
                        .local_get(vector_ptr)
                        .i32_const(4)
                        .binop(BinaryOp::I32Add)
                        .local_set(vector_ptr);

                    loop_
                        .local_get(i)
                        .i32_const(1)
                        .binop(BinaryOp::I32Add)
                        .local_tee(i);

                    loop_.local_get(len).binop(BinaryOp::I32LtU).br_if(loop_id);
                    Ok(())
                })();
            });
        }

        // Enums are treated as vectors of u8
        IntermediateType::IEnum { module_id, index } => {
            let enum_ = compilation_ctx.get_enum_by_index(module_id, *index)?;

            if !enum_.is_simple {
                return Err(NativeFunctionError::EmitFunctionInvalidEventField(
                    IntermediateType::IVector(Arc::new(inner.clone())),
                ));
            }

            let value = module.locals.add(ValType::I32);
            let writer_pointer = module.locals.add(ValType::I32);
            builder.i32_const(0).local_set(i);

            builder.loop_(None, |loop_| {
                let loop_id = loop_.id();

                inner_result = (|| {
                    loop_
                        .local_get(vector_ptr)
                        .load(
                            compilation_ctx.memory_id,
                            LoadKind::I32 { atomic: false },
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        )
                        .load(
                            compilation_ctx.memory_id,
                            LoadKind::I32 { atomic: false },
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        )
                        .local_set(value);

                    // Allocate 32 bytes for the encoded data
                    loop_
                        .i32_const(32)
                        .call(compilation_ctx.allocator)
                        .local_set(writer_pointer);

                    IntermediateType::IU8.add_pack_instructions(
                        loop_,
                        module,
                        value,
                        writer_pointer,
                        writer_pointer,
                        compilation_ctx,
                        None,
                        None,
                    )?;

                    loop_
                        .local_get(vector_ptr)
                        .i32_const(4)
                        .binop(BinaryOp::I32Add)
                        .local_set(vector_ptr);

                    // increment i
                    loop_
                        .local_get(i)
                        .i32_const(1)
                        .binop(BinaryOp::I32Add)
                        .local_tee(i);

                    loop_.local_get(len).binop(BinaryOp::I32LtU).br_if(loop_id);

                    Ok(())
                })();
            });
        }
        _ => {
            return Err(NativeFunctionError::EmitFunctionInvalidEventField(
                IntermediateType::IVector(Arc::new(inner.clone())),
            ));
        }
    }

    inner_result?;

    Ok(())
}

/// Encodes a struct following the ABI spec for events
///
/// https://docs.soliditylang.org/en/latest/abi-spec.html#indexed-event-encoding
fn add_encode_indexed_struct_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
    struct_: &IStruct,
    struct_ptr: LocalId,
) -> Result<(), NativeFunctionError> {
    for (index, field) in struct_.fields.iter().enumerate() {
        match field {
            // If the data is "simple" we just concatenate things contigously
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64
            | IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::IAddress => {
                let writer_pointer = module.locals.add(ValType::I32);
                let value = module.locals.add(ValType::try_from(field)?);

                builder.local_get(struct_ptr).load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: index as u32 * 4,
                    },
                );

                if field.is_stack_type()? {
                    builder
                        .load(
                            compilation_ctx.memory_id,
                            field.load_kind()?,
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        )
                        .local_set(value);
                } else {
                    builder.local_set(value);
                }

                // Allocate 32 bytes for the encoded data
                builder
                    .i32_const(32)
                    .call(compilation_ctx.allocator)
                    .local_set(writer_pointer);

                field.add_pack_instructions(
                    builder,
                    module,
                    value,
                    writer_pointer,
                    writer_pointer,
                    compilation_ctx,
                    None,
                    None,
                )?;
            }
            IntermediateType::IVector(inner) => {
                let value = module.locals.add(ValType::I32);

                builder
                    .local_get(struct_ptr)
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: index as u32 * 4,
                        },
                    )
                    .local_set(value);

                add_encode_indexed_vector_instructions(
                    module,
                    builder,
                    compilation_ctx,
                    runtime_error_data,
                    inner,
                    value,
                )?;
            }
            IntermediateType::IStruct {
                module_id,
                index: struct_index,
                ..
            } if String_::is_vm_type(module_id, *struct_index, compilation_ctx)? => {
                let value = module.locals.add(ValType::I32);
                builder
                    .local_get(struct_ptr)
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: index as u32 * 4,
                        },
                    )
                    .local_set(value);

                add_encode_indexed_string(module, builder, compilation_ctx, value, true)?;
            }
            IntermediateType::IStruct {
                module_id,
                index: struct_index,
                ..
            }
            | IntermediateType::IGenericStructInstance {
                module_id,
                index: struct_index,
                ..
            } => {
                let value = module.locals.add(ValType::I32);
                let child_struct = compilation_ctx.get_struct_by_index(module_id, *struct_index)?;

                let child_struct =
                    if let IntermediateType::IGenericStructInstance { types, .. } = field {
                        &child_struct.instantiate(types)
                    } else {
                        child_struct
                    };

                builder
                    .local_get(struct_ptr)
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: index as u32 * 4,
                        },
                    )
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_set(value);

                add_encode_indexed_struct_instructions(
                    module,
                    builder,
                    compilation_ctx,
                    runtime_error_data,
                    child_struct,
                    value,
                )?;
            }
            IntermediateType::IEnum {
                module_id,
                index: enum_index,
            } => {
                let value = module.locals.add(ValType::I32);
                let writer_pointer = module.locals.add(ValType::I32);
                let child_enum = compilation_ctx.get_enum_by_index(module_id, *enum_index)?;

                if !child_enum.is_simple {
                    return Err(NativeFunctionError::EmitFunctionInvalidEventField(
                        field.clone(),
                    ));
                }

                builder
                    .local_get(struct_ptr)
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: index as u32 * 4,
                        },
                    )
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32_8 {
                            kind: ExtendedLoad::ZeroExtend,
                        },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_set(value);

                // Allocate 32 bytes for the encoded data
                builder
                    .i32_const(32)
                    .call(compilation_ctx.allocator)
                    .local_set(writer_pointer);

                IntermediateType::IU8.add_pack_instructions(
                    builder,
                    module,
                    value,
                    writer_pointer,
                    writer_pointer,
                    compilation_ctx,
                    None,
                    None,
                )?;
            }
            _ => {
                return Err(NativeFunctionError::EmitFunctionInvalidEventField(
                    field.clone(),
                ));
            }
        }
    }

    Ok(())
}

/// Encodes a struct following the ABI spec for events
///
/// https://docs.soliditylang.org/en/latest/abi-spec.html#indexed-event-encoding
fn add_encode_indexed_string(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    string_ptr: LocalId,
    padded: bool,
) -> Result<(), NativeFunctionError> {
    let writer_pointer = module.locals.add(ValType::I32);

    // String in move have the following form:
    // public struct String has copy, drop, store {
    //   bytes: vector<u8>,
    // }
    //
    // So we need to perform a load first to get to the inner vector
    builder
        .local_get(string_ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(string_ptr);

    let len = IntermediateType::IU32.add_load_memory_to_local_instructions(
        module,
        builder,
        string_ptr,
        compilation_ctx.memory_id,
    )?;

    if padded {
        builder
            .local_get(len)
            .i32_const(31)
            .binop(BinaryOp::I32Add)
            .i32_const(!31)
            .binop(BinaryOp::I32And)
            .call(compilation_ctx.allocator)
            .local_set(writer_pointer);
    } else {
        builder
            .local_get(len)
            .call(compilation_ctx.allocator)
            .local_set(writer_pointer);
    }

    // Set the local to point to the first element
    builder.skip_vec_header(string_ptr).local_set(string_ptr);

    // Loop through the vector values
    let i = module.locals.add(ValType::I32);
    builder.i32_const(0).local_set(i);
    builder.loop_(None, |loop_block| {
        let loop_block_id = loop_block.id();

        loop_block
            .local_get(writer_pointer)
            .local_get(string_ptr)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )
            .store(
                compilation_ctx.memory_id,
                StoreKind::I32_8 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

        // Increment the string pointer by 1 byte to point to the next u8 element
        loop_block
            .local_get(string_ptr)
            .i32_const(1)
            .binop(BinaryOp::I32Add)
            .local_set(string_ptr);

        // Increment the writer pointer by 1 byte to point to the next u8 element
        loop_block
            .local_get(writer_pointer)
            .i32_const(1)
            .binop(BinaryOp::I32Add)
            .local_set(writer_pointer);

        // Increment i
        loop_block
            .local_get(i)
            .i32_const(1)
            .binop(BinaryOp::I32Add)
            .local_tee(i);

        loop_block
            .local_get(len)
            .binop(BinaryOp::I32LtU)
            .br_if(loop_block_id);
    });

    Ok(())
}
