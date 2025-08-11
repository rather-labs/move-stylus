//! Here is implemented the function that prepares the data to be saved in storage.

use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, ExtendedLoad, LoadKind, MemArg, StoreKind},
};

use crate::{
    CompilationContext,
    data::{DATA_SLOT_DATA_PTR_OFFSET, DATA_U256_ONE_OFFSET},
    hostio::host_functions::storage_load_bytes32,
    runtime::RuntimeFunction,
    translation::intermediate_types::{
        IntermediateType,
        heap_integers::{IU128, IU256},
        structs::IStruct,
    },
};

/// This function adds the instruction to save in storage a structure
pub fn add_decode_storage_struct_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    slot_ptr: LocalId,
    struct_: &IStruct,
) -> LocalId {
    let (storage_load, _) = storage_load_bytes32(module);

    let struct_ptr = module.locals.add(ValType::I32);

    // Locals
    let field_ptr = module.locals.add(ValType::I32);
    let val_64 = module.locals.add(ValType::I64);
    let val_32 = module.locals.add(ValType::I32);

    // Allocate space for the struct
    builder
        .i32_const(struct_.heap_size as i32)
        .call(compilation_ctx.allocator)
        .local_set(struct_ptr);

    // Load data from slot
    builder
        .local_get(slot_ptr)
        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
        .call(storage_load);

    let mut read_bytes_in_slot = 0;
    for (index, field) in struct_.fields.iter().enumerate() {
        let field_size = field_size(field);
        if read_bytes_in_slot + field_size > 32 {
            let swap_256_fn = RuntimeFunction::SwapI256Bytes.get(module, Some(compilation_ctx));

            // BE to LE ptr so we can make the addition
            builder
                .local_get(slot_ptr)
                .local_get(slot_ptr)
                .call(swap_256_fn);

            // Add one to slot
            let add_u256_fn = RuntimeFunction::HeapIntSum.get(module, Some(compilation_ctx));
            builder
                .local_get(slot_ptr)
                .i32_const(DATA_U256_ONE_OFFSET)
                .local_get(slot_ptr)
                .i32_const(32)
                .call(add_u256_fn)
                .local_set(slot_ptr);

            // LE to BE ptr so we can use the storage function
            builder
                .local_get(slot_ptr)
                .local_get(slot_ptr)
                .call(swap_256_fn);

            // Load the slot data
            builder
                .local_get(slot_ptr)
                .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                .call(storage_load);

            read_bytes_in_slot = field_size;
        } else {
            read_bytes_in_slot += field_size;
        }

        match field {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64 => {
                let data_size = field.stack_data_size();
                let (val, store_kind, swap_fn) = if data_size == 8 {
                    let swap_fn = RuntimeFunction::SwapI64Bytes.get(module, None);
                    (val_64, StoreKind::I64 { atomic: false }, swap_fn)
                } else {
                    let swap_fn = RuntimeFunction::SwapI32Bytes.get(module, None);
                    (val_32, StoreKind::I32 { atomic: false }, swap_fn)
                };

                // Create a pointer for the value
                builder
                    .i32_const(data_size as i32)
                    .call(compilation_ctx.allocator)
                    .local_tee(field_ptr);

                // Read the value from the slot
                let load_kind = match field_size {
                    1 => LoadKind::I32_8 {
                        kind: ExtendedLoad::ZeroExtend,
                    },
                    2 => LoadKind::I32_16 {
                        kind: ExtendedLoad::ZeroExtend,
                    },
                    4 => LoadKind::I32 { atomic: false },
                    8 => LoadKind::I64 { atomic: false },
                    _ => panic!("invalid field size {field_size} for type {field:?}"),
                };

                // Read the value and transform it to LE
                builder
                    .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                    .load(
                        compilation_ctx.memory_id,
                        load_kind,
                        MemArg {
                            align: 0,
                            offset: 32 - read_bytes_in_slot,
                        },
                    )
                    .local_tee(val)
                    .call(swap_fn)
                    .local_set(val);

                // If the field size are less than 4 or 8 bytes we need to shift them before
                // saving
                if field_size == 1 {
                    builder
                        .local_get(val)
                        .i32_const(24)
                        .binop(BinaryOp::I32ShrU)
                        .local_set(val);
                } else if field_size == 2 {
                    builder
                        .local_get(val)
                        .i32_const(16)
                        .binop(BinaryOp::I32ShrU)
                        .local_set(val);
                }

                // Save it to the struct
                builder.local_get(val).store(
                    compilation_ctx.memory_id,
                    store_kind,
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            }
            IntermediateType::IU128 => {
                // Create a pointer for the value
                builder
                    .i32_const(IU128::HEAP_SIZE)
                    .call(compilation_ctx.allocator)
                    .local_tee(field_ptr);

                // Source address (plus offset)
                builder
                    .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                    .i32_const(32 - read_bytes_in_slot as i32)
                    .binop(BinaryOp::I32Add);

                // Number of bytes to copy
                builder.i32_const(IU128::HEAP_SIZE);

                // Copy the chunk of memory
                builder.memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

                let swap_fn = RuntimeFunction::SwapI128Bytes.get(module, Some(compilation_ctx));

                // Transform it to LE
                builder
                    .local_get(field_ptr)
                    .local_get(field_ptr)
                    .call(swap_fn);
            }
            IntermediateType::IU256 => {
                // Create a pointer for the value
                builder
                    .i32_const(IU256::HEAP_SIZE)
                    .call(compilation_ctx.allocator)
                    .local_tee(field_ptr);

                // Source address (plus offset)
                builder.i32_const(DATA_SLOT_DATA_PTR_OFFSET);

                // Number of bytes to copy
                builder.i32_const(32);

                // Copy the chunk of memory
                builder.memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

                let swap_fn = RuntimeFunction::SwapI256Bytes.get(module, Some(compilation_ctx));

                // Transform it to LE
                builder
                    .local_get(field_ptr)
                    .local_get(field_ptr)
                    .call(swap_fn);
            }
            IntermediateType::IAddress | IntermediateType::ISigner => {
                // Create a pointer for the value
                builder
                    .i32_const(32)
                    .call(compilation_ctx.allocator)
                    .local_tee(field_ptr);

                // Source address (plus offset)
                builder.i32_const(DATA_SLOT_DATA_PTR_OFFSET);

                // Number of bytes to copy
                builder.i32_const(32);

                // Copy the chunk of memory
                builder.memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);
            }
            _ => todo!(),
        };

        // Save the ptr value to the struct
        builder.local_get(struct_ptr).local_get(field_ptr).store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: index as u32 * 4,
            },
        );
    }

    struct_ptr
}

fn field_size(field: &IntermediateType) -> u32 {
    match field {
        IntermediateType::IBool | IntermediateType::IU8 | IntermediateType::IEnum(_) => 1,
        IntermediateType::IU16 => 2,
        IntermediateType::IU32 => 4,
        IntermediateType::IU64 => 8,
        IntermediateType::IU128 => 16,
        IntermediateType::IU256 | IntermediateType::IAddress | IntermediateType::ISigner => 32,
        // Dynamic data occupies the whole slot, but the data is saved somewhere else
        IntermediateType::IVector(_)
        | IntermediateType::IGenericStructInstance { .. }
        | IntermediateType::IStruct { .. } => 32,

        IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
            panic!("found reference inside struct")
        }
        IntermediateType::ITypeParameter(_) => {
            panic!("cannot know if a type parameter is dynamic, expected a concrete type");
        }
        IntermediateType::IExternalUserData { .. } => todo!(),
    }
}
