use std::process::Child;

use super::NativeFunction;
use crate::{
    CompilationContext,
    data::{DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET, DATA_STORAGE_OBJECT_OWNER_OFFSET},
    get_generic_function_name,
    hostio::host_functions::native_keccak256,
    runtime::RuntimeFunction,
    storage::encoding::add_encode_and_save_into_storage_struct_instructions,
    translation::intermediate_types::{
        IntermediateType,
        address::IAddress,
        heap_integers::{IU128, IU256},
    },
    wasm_builder_extensions::WasmBuilderExtension,
};

use walrus::{
    FunctionBuilder, FunctionId, InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

pub fn add_child_object_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
) -> FunctionId {
    let name = get_generic_function_name(NativeFunction::NATIVE_ADD_CHILD_OBJECT, &[itype]);
    if let Some(function) = module.funcs.by_name(&name) {
        return function;
    };

    let get_id_bytes_ptr_fn = RuntimeFunction::GetIdBytesPtr.get(module, Some(compilation_ctx));
    let write_object_slot_fn = RuntimeFunction::WriteObjectSlot.get(module, Some(compilation_ctx));
    let save_struct_into_storage_fn =
        RuntimeFunction::EncodeAndSaveInStorage.get_generic(module, compilation_ctx, &[itype]);

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32], &[]);

    let mut builder = function.name(name).func_body();

    // Arguments
    let parent_address = module.locals.add(ValType::I32);
    let child_ptr = module.locals.add(ValType::I32);

    // Calculate the destiny slot
    builder
        .local_get(parent_address)
        .local_get(child_ptr)
        .call(get_id_bytes_ptr_fn)
        .call(write_object_slot_fn);

    // Save the field into storage
    builder
        .local_get(child_ptr)
        .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
        .call(save_struct_into_storage_fn);

    function.finish(vec![parent_address, child_ptr], &mut module.funcs)
}

pub fn add_borrow_object_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
) -> FunctionId {
    let name = get_generic_function_name(NativeFunction::NATIVE_BORROW_CHILD_OBJECT, &[itype]);
    if let Some(function) = module.funcs.by_name(&name) {
        return function;
    };

    let write_object_slot_fn = RuntimeFunction::WriteObjectSlot.get(module, Some(compilation_ctx));
    let decode_and_read_from_storage_fn =
        RuntimeFunction::DecodeAndReadFromStorage.get_generic(module, compilation_ctx, &[itype]);

    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );

    let mut builder = function.name(name).func_body();

    // Arguments
    let parent_uid = module.locals.add(ValType::I32);
    let child_id = module.locals.add(ValType::I32);

    // Calculate the destiny slot
    builder
        .local_get(parent_uid)
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
        .local_tee(parent_uid)
        .local_get(child_id)
        .call(write_object_slot_fn);

    // Write the owner
    builder
        .i32_const(DATA_STORAGE_OBJECT_OWNER_OFFSET)
        .local_get(parent_uid)
        .i32_const(32)
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    let tmp = module.locals.add(ValType::I32);

    // Read from storage
    builder
        .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
        .call(decode_and_read_from_storage_fn)
        .local_tee(tmp);

    function.finish(vec![parent_uid, child_id], &mut module.funcs)
}

/// Computes a keccak256 hash from:
/// - parent address (32 bytes)
/// - key (variable size)
/// - Key type name
///
/// Arguments
/// * `parent_address` - i32 pointer to the parent address in memory
/// * `key` - i32 pointer to the key in memory
///
/// Returns
/// * i32 pointer to the resulting hash in memory
pub fn add_hash_type_and_key_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
) -> FunctionId {
    let name = get_generic_function_name(NativeFunction::NATIVE_HASH_TYPE_AND_KEY, &[itype]);
    if let Some(function) = module.funcs.by_name(&name) {
        return function;
    };

    let (native_keccak, _) = native_keccak256(module);

    // Arguments
    let parent_address = module.locals.add(ValType::I32);
    let (key, valtype) = if itype == &IntermediateType::IU64 {
        (module.locals.add(ValType::I64), ValType::I64)
    } else {
        (module.locals.add(ValType::I32), ValType::I32)
    };

    let mut function =
        FunctionBuilder::new(&mut module.types, &[ValType::I32, valtype], &[ValType::I32]);

    let mut builder = function.name(name).func_body();

    // Locals
    let data_start = module.locals.add(ValType::I32);
    let result_ptr = module.locals.add(ValType::I32);

    // Fist we allocate space for the address
    builder
        .i32_const(IAddress::HEAP_SIZE)
        .call(compilation_ctx.allocator)
        .local_set(data_start);

    builder
        .local_get(data_start)
        .local_get(parent_address)
        .i32_const(IAddress::HEAP_SIZE)
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    // Copy the data after the parent addresss
    copy_data_to_memory(&mut builder, compilation_ctx, module, itype, key);

    let type_name = itype.get_name(compilation_ctx);

    for chunk in type_name.as_bytes() {
        builder.i32_const(1).call(compilation_ctx.allocator);

        builder.i32_const(*chunk as i32).store(
            compilation_ctx.memory_id,
            StoreKind::I32_8 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );
    }

    builder.local_get(data_start);

    // Call allocator to get the end of the data to Hash and substract the start to get the length
    builder
        .i32_const(0)
        .call(compilation_ctx.allocator)
        .local_get(data_start)
        .binop(BinaryOp::I32Sub);

    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_tee(result_ptr);

    builder.call(native_keccak).local_get(result_ptr);

    function.finish(vec![parent_address, key], &mut module.funcs)
}

fn copy_data_to_memory(
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    module: &mut Module,
    itype: &IntermediateType,
    data: LocalId,
) {
    let load_value_to_stack = |field: &IntermediateType, builder: &mut InstrSeqBuilder<'_>| {
        if field.stack_data_size() == 8 {
            builder.load(
                compilation_ctx.memory_id,
                LoadKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );
        } else {
            builder.load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );
        }
    };
    // Copy the data after the parent addresss
    match itype {
        IntermediateType::IAddress => {
            builder
                .i32_const(IAddress::HEAP_SIZE)
                .call(compilation_ctx.allocator);

            builder
                .local_get(data)
                .i32_const(IAddress::HEAP_SIZE)
                .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);
        }
        // 4 bytes numbers should be in the stack
        IntermediateType::IBool | IntermediateType::IU8 => {
            builder.i32_const(1).call(compilation_ctx.allocator);

            builder.local_get(data).store(
                compilation_ctx.memory_id,
                StoreKind::I32_8 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );
        }
        IntermediateType::IU16 => {
            builder.i32_const(2).call(compilation_ctx.allocator);

            builder.local_get(data).store(
                compilation_ctx.memory_id,
                StoreKind::I32_16 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );
        }
        IntermediateType::IU32 => {
            builder.i32_const(4).call(compilation_ctx.allocator);

            builder.local_get(data).store(
                compilation_ctx.memory_id,
                StoreKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );
        }
        IntermediateType::IU64 => {
            builder.i32_const(8).call(compilation_ctx.allocator);

            builder.local_get(data).store(
                compilation_ctx.memory_id,
                StoreKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );
        }
        IntermediateType::IU128 => {
            builder
                .i32_const(IU128::HEAP_SIZE)
                .call(compilation_ctx.allocator);

            builder
                .local_get(data)
                .i32_const(IU128::HEAP_SIZE)
                .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);
        }
        IntermediateType::IU256 => {
            builder
                .i32_const(IU256::HEAP_SIZE)
                .call(compilation_ctx.allocator);

            builder
                .local_get(data)
                .i32_const(IU256::HEAP_SIZE)
                .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);
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

            let struct_ = match itype {
                IntermediateType::IGenericStructInstance { types, .. } => {
                    &struct_.instantiate(types)
                }
                _ => struct_,
            };

            let field_data_32 = module.locals.add(ValType::I32);
            let field_data_64 = module.locals.add(ValType::I64);

            for (index, field) in struct_.fields.iter().enumerate() {
                let field_data = if field == &IntermediateType::IU64 {
                    field_data_64
                } else {
                    field_data_32
                };

                builder.local_get(data).load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: index as u32 * 4,
                    },
                );

                if field.is_stack_type() {
                    load_value_to_stack(field, builder);
                }

                builder.local_set(field_data);

                copy_data_to_memory(builder, compilation_ctx, module, field, field_data);
            }
        }
        IntermediateType::IVector(inner) => {
            let len = module.locals.add(ValType::I32);
            let i = module.locals.add(ValType::I32);
            builder
                .local_get(data)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .local_set(len);

            let (field_data, load_kind, element_multiplier) = if **inner == IntermediateType::IU64 {
                (
                    module.locals.add(ValType::I64),
                    LoadKind::I64 { atomic: false },
                    8,
                )
            } else {
                (
                    module.locals.add(ValType::I32),
                    LoadKind::I32 { atomic: false },
                    4,
                )
            };

            builder.i32_const(0).local_set(i);
            builder.skip_vec_header(data).local_set(data);

            builder.block(None, |block| {
                let block_id = block.id();
                block.loop_(None, |loop_| {
                    let loop_id = loop_.id();

                    // Load the element pointer from the vector data
                    loop_
                        .local_get(data)
                        .i32_const(element_multiplier)
                        .local_get(i)
                        .binop(BinaryOp::I32Mul)
                        .binop(BinaryOp::I32Add)
                        .load(
                            compilation_ctx.memory_id,
                            load_kind,
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        )
                        .local_set(field_data);

                    copy_data_to_memory(loop_, compilation_ctx, module, inner, field_data);

                    // If we reach the last element, we exit
                    loop_
                        .local_get(i)
                        .local_get(len)
                        .i32_const(1)
                        .binop(BinaryOp::I32Sub)
                        .binop(BinaryOp::I32Eq)
                        .br_if(block_id);

                    // Else, increment i and continue the loop
                    loop_
                        .local_get(i)
                        .i32_const(1)
                        .binop(BinaryOp::I32Add)
                        .local_set(i)
                        .br(loop_id);
                });
            });
        }

        _ => {
            panic!(
                r#"there was an error linking "{}" function, unsupported key type {itype:?}"#,
                NativeFunction::NATIVE_HASH_TYPE_AND_KEY
            );
        }
    }
}
