//! Here is implemented the function that prepares the data to be saved in storage.

use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use crate::{
    CompilationContext,
    hostio::{
        self,
        host_functions::{block_number, storage_cache_bytes32, storage_flush_cache},
    },
    runtime::RuntimeFunction,
    translation::intermediate_types::{IntermediateType, structs::IStruct},
};

/// This function adds the instruction to save in storage a structure
pub fn store(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    struct_ptr: LocalId,
    struct_: &IStruct,
) -> LocalId {
    let mut size = 0;
    let mut slots = 1;

    let (storage_cache, _) = storage_cache_bytes32(module);
    let (storage_flush_cache, _) = storage_flush_cache(module);

    let slot_ptr = module.locals.add(ValType::I32);
    let slot_data_ptr = module.locals.add(ValType::I32);
    let val_32 = module.locals.add(ValType::I32);
    let val_64 = module.locals.add(ValType::I64);

    // At the moment we use slot 0 for testing
    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_set(slot_ptr);

    // Allocate 32 bytes to save the current slot data
    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_set(slot_data_ptr);

    for (index, field) in struct_.fields.iter().enumerate() {
        let field_size = field_size(&field);
        if size + field_size >= 32 {
            // Wipe the data
            builder
                .local_get(slot_data_ptr)
                .i32_const(0)
                .memory_fill(compilation_ctx.memory_id);

            slots += 1;
            size = field_size;
        } else {
            size += field_size;
        }

        // Load field's intermediate pointer
        builder.local_get(struct_ptr).load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: index as u32 * 4,
            },
        );

        match field {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64 => {
                let (val, load_kind, swap_fn) = if field.stack_data_size() == 8 {
                    let swap_fn = RuntimeFunction::SwapI64Bytes.get(module, None);
                    (val_64, LoadKind::I64 { atomic: false }, swap_fn)
                } else {
                    let swap_fn = RuntimeFunction::SwapI32Bytes.get(module, None);
                    (val_32, LoadKind::I32 { atomic: false }, swap_fn)
                };

                builder
                    .load(
                        compilation_ctx.memory_id,
                        load_kind,
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_tee(val);

                // Convert the value to big endian
                builder.call(swap_fn).local_set(val);

                // We need to shift the swapped bytes to the left because WASM is little endian. If we try
                // to write a 16 bits number contained in a 32 bits number, without shifting, it will write
                // the zeroed part.
                // This only needs to be done for 32 bits (4 bytes) numbers

                if field.stack_data_size() == 4 {
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
                }

                let store_kind = if field_size == 1 {
                    StoreKind::I32_8 { atomic: false }
                } else if field_size == 2 {
                    StoreKind::I32_16 { atomic: false }
                } else if field_size == 4 {
                    StoreKind::I32 { atomic: false }
                } else {
                    StoreKind::I64 { atomic: false }
                };

                // Save the value in slot data
                builder.local_get(slot_data_ptr).local_get(val).store(
                    compilation_ctx.memory_id,
                    store_kind,
                    MemArg {
                        align: 0,
                        offset: 32 - size,
                    },
                );
            }
            _ => {}
        };
    }

    builder
        .local_get(slot_ptr)
        .local_get(slot_data_ptr)
        .call(storage_cache);

    builder.i32_const(1).call(storage_flush_cache);

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
