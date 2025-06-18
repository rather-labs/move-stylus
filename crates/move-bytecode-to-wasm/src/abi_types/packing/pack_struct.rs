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
        let struct_ptr = local;

        println!(
            "=== > {}",
            struct_.solidity_abi_encode_size(compilation_ctx)
        );

        // If the struct is dynamic, the space allocated for the struct is only 32 bytes long and we
        // need to save in calldata_reference_pointer the value pointing to the packed struct.
        //
        // If the struct is static, the space for packing it is already allocated, we just need to
        // pack its values
        if struct_.solidity_abi_encode_is_dynamic(compilation_ctx) {
            // Allocate memory for the packed value.
            block
                .i32_const(struct_.solidity_abi_encode_size(compilation_ctx) as i32)
                .call(compilation_ctx.allocator)
                .local_tee(calldata_reference_pointer)
                .local_set(writer_pointer);
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

            // Pack field
            field.add_pack_instructions(
                block,
                module,
                field_local,
                writer_pointer,
                calldata_reference_pointer,
                compilation_ctx,
            );

            block
                .i32_const(32)
                .local_get(writer_pointer)
                .binop(BinaryOp::I32Add)
                .local_set(writer_pointer);
        }

        // block.local_get(calldata_reference_pointer);
    }
}
