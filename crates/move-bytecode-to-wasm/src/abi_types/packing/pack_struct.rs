use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg},
};

use crate::{
    CompilationContext,
    abi_types::packing::pack_native_int::pack_i32_type_instructions,
    translation::intermediate_types::{IntermediateType, structs::IStruct},
};

use super::Packable;

impl IStruct {
    #[allow(clippy::too_many_arguments)]
    pub fn add_pack_instructions(
        index: u16,
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        local: LocalId,
        writer_pointer: LocalId,
        calldata_reference_pointer: LocalId,
        compilation_ctx: &CompilationContext,
        base_calldata_reference_pointer: Option<LocalId>,
    ) {
        let struct_ = compilation_ctx.get_struct_by_index(index).unwrap();
        let val_32 = module.locals.add(ValType::I32);
        let val_64 = module.locals.add(ValType::I64);
        let struct_ptr = local;
        let reference_value = module.locals.add(ValType::I32);

        let data_ptr = module.locals.add(ValType::I32);
        let inner_data_reference = module.locals.add(ValType::I32);

        // If base_calldata_reference_ptr is Some(_), means we are packing an struct inside a
        // struct and that the struct is dynamic.
        // base_calldata_reference_pointer is the reference pointer to the original value, and it
        // is used to calulcate the offset where the struct will be allocated in the parent struct.
        // The calculated offset will be written in the place where the struct should be.
        if let Some(base_calldata_reference_ptr) = base_calldata_reference_pointer {
            // Allocate memory for the packed value. Set the writer pointer at the beginning, since
            // we are going to pack the values from there
            block
                .i32_const(struct_.solidity_abi_encode_size(compilation_ctx) as i32)
                .call(compilation_ctx.allocator)
                .local_tee(data_ptr)
                .local_tee(inner_data_reference);

            // The pointer in the packed data must be relative to the calldata_reference_pointer,
            // so we substract calldata_reference_pointer from the writer_pointer
            block
                .local_get(base_calldata_reference_ptr)
                .binop(BinaryOp::I32Sub)
                .local_set(reference_value);

            // The result is saved where calldata_reference_pointer is pointing at, the value will
            // be the address where the struct  values are packed, using as origin
            // calldata_reference_pointer
            pack_i32_type_instructions(
                block,
                module,
                compilation_ctx.memory_id,
                reference_value,
                writer_pointer,
            );

            // block.local_get(data_ptr).local_set(writer_pointer);
        }

        // Load the value to be written in the calldata, if it is a stack value we need to double
        // reference a pointer, otherwise we read the pointer and leave the stack value in the
        // stack
        for (index, field) in struct_.fields.iter().enumerate() {
            // Load field's intermediate pointer
            block.local_get(struct_ptr).load(
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
                    let (val, load_kind) = if field.stack_data_size() == 8 {
                        (val_64, LoadKind::I64 { atomic: false })
                    } else {
                        (val_32, LoadKind::I32 { atomic: false })
                    };

                    block
                        .load(
                            compilation_ctx.memory_id,
                            load_kind,
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        )
                        .local_set(val);

                    val
                }
                _ => {
                    block.local_set(val_32);
                    val_32
                }
            };

            // If base_calldata_reference_pointer is none, means we are not packing this struct
            // dynamically, so, we can set data_ptr as the writer pointer and the
            // inner_data_reference as the root reference pointer
            if base_calldata_reference_pointer.is_none() {
                block.local_get(writer_pointer).local_set(data_ptr);
                block
                    .local_get(calldata_reference_pointer)
                    .local_set(inner_data_reference);
            }

            // If the field to pack is a struct, it will be packed dynamically, that means, in the
            // current offset of writer pointer, we are going to write the offset where we can find
            // the struct
            if let IntermediateType::IStruct(i) = field {
                let child_struct = compilation_ctx.get_struct_by_index(*i).unwrap();
                if child_struct.solidity_abi_encode_is_dynamic(compilation_ctx) {
                    IStruct::add_pack_instructions(
                        *i,
                        block,
                        module,
                        field_local,
                        data_ptr,
                        inner_data_reference,
                        compilation_ctx,
                        Some(calldata_reference_pointer),
                    )
                } else {
                    field.add_pack_instructions(
                        block,
                        module,
                        field_local,
                        data_ptr,
                        inner_data_reference,
                        compilation_ctx,
                    );
                }
            } else {
                field.add_pack_instructions(
                    block,
                    module,
                    field_local,
                    data_ptr,
                    inner_data_reference,
                    compilation_ctx,
                );
            }

            // If base_calldata_reference_pointer is none, we are packing the struct in place, so
            // we move the writer_pointer, otherwise, we allocated memory to pack it somewhere else
            // (because it is dynamic), data_ptr is the offset where data will be written.
            let pointer_to_update = if base_calldata_reference_pointer.is_none() {
                writer_pointer
            } else {
                data_ptr
            };

            block
                .i32_const(field.encoded_size(compilation_ctx) as i32)
                .local_get(pointer_to_update)
                .binop(BinaryOp::I32Add)
                .local_set(pointer_to_update);
        }
    }
}
