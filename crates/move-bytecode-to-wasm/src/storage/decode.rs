//! Here is implemented the function that prepares the data to be saved in storage.

use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, ExtendedLoad, LoadKind, MemArg, StoreKind},
};

use crate::{
    CompilationContext,
    hostio::{
        self,
        host_functions::{
            block_number, storage_cache_bytes32, storage_flush_cache, storage_load_bytes32,
        },
    },
    runtime::RuntimeFunction,
    translation::intermediate_types::{IntermediateType, structs::IStruct},
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
    let slot_data_ptr = module.locals.add(ValType::I32);

    // Locals
    let field_ptr = module.locals.add(ValType::I32);
    let val_64 = module.locals.add(ValType::I64);
    let val_32 = module.locals.add(ValType::I32);

    let offset = module.locals.add(ValType::I32);

    // Allocate space for the struct
    builder
        .i32_const(struct_.heap_size as i32)
        .call(compilation_ctx.allocator)
        .local_set(struct_ptr);

    // Allocate space for reading the slot
    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_set(slot_data_ptr);

    // Load data from slot
    builder
        .local_get(slot_ptr)
        .local_get(slot_data_ptr)
        .call(storage_load);

    let mut read_bytes_in_slot = 0;
    for (index, field) in struct_.fields.iter().enumerate() {
        let field_size = field_size(&field);
        if read_bytes_in_slot + field_size > 32 {
            read_bytes_in_slot = field_size;
            builder.i32_const(field_size as i32).local_set(offset);
            // TODO: Move to the next slot and save it in slot_ptr
            builder
                .local_get(slot_ptr)
                .local_get(slot_data_ptr)
                .call(storage_load);
        } else {
            builder
                .i32_const(32)
                .i32_const(field_size as i32)
                .binop(BinaryOp::I32Sub)
                .local_set(offset);

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
                    .local_get(slot_data_ptr)
                    .load(
                        compilation_ctx.memory_id,
                        load_kind,
                        MemArg {
                            align: 0,
                            offset: read_bytes_in_slot,
                        },
                    )
                    .local_tee(val)
                    .call(swap_fn)
                    .local_set(val);

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
            IntermediateType::IU128 => {}
            IntermediateType::IU256 | IntermediateType::IAddress | IntermediateType::ISigner => {}
            _ => todo!(),
        };
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
