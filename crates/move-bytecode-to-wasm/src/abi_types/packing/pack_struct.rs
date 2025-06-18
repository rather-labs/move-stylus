use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg},
};

use crate::{
    CompilationContext,
    translation::intermediate_types::{IntermediateType, structs::IStruct},
};

use super::Packable;

impl IStruct {
    pub fn add_pack_instructions(
        index: u16,
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        local: LocalId,
        writer_pointer: LocalId,
        calldata_reference_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) {
        let struct_ = compilation_ctx.get_struct_by_index(index).unwrap();
        let val_32 = module.locals.add(ValType::I32);
        let val_64 = module.locals.add(ValType::I64);
        let packed_struct_ptr = module.locals.add(ValType::I32);
        let write_data_ptr = module.locals.add(ValType::I32);
        let struct_ptr = local;

        // Allocate memory for the packed value. To calculate the size of the allocation we divide
        // the struct heap size by 4 to compute the number of fields, after that we multiply it by
        // 32 because each field will occupy 32 bytes
        block
            .i32_const(struct_.heap_size as i32 / 4 * 32)
            .call(compilation_ctx.allocator)
            .local_tee(packed_struct_ptr)
            .local_set(write_data_ptr);

        if struct_.solidity_abi_encode_is_dynamic(compilation_ctx) {
            todo!()
        }

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

            // Unpack field
            field.add_pack_instructions(
                block,
                module,
                field_local,
                write_data_ptr,
                packed_struct_ptr,
                compilation_ctx,
            );

            block
                .i32_const(32)
                .local_get(write_data_ptr)
                .binop(BinaryOp::I32Add)
                .local_set(write_data_ptr);
        }

        block.local_get(packed_struct_ptr);
    }
}
