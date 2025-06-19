use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use crate::{
    CompilationContext,
    abi_types::packing::pack_native_int::pack_i32_type_instructions,
    runtime::RuntimeFunction,
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
        let print_memory_from = module.imports.get_func("", "print_memory_from").unwrap();
        let print_i32 = module.imports.get_func("", "print_i32").unwrap();

        let struct_ = compilation_ctx.get_struct_by_index(index).unwrap();
        let val_32 = module.locals.add(ValType::I32);
        let val_64 = module.locals.add(ValType::I64);
        let struct_ptr = local;

        let writer_ptr = module.locals.add(ValType::I32);

        println!(
            "=== > {}",
            struct_.solidity_abi_encode_size(compilation_ctx)
        );

        block.local_get(calldata_reference_pointer).call(print_i32);
        block.local_get(writer_pointer).call(print_i32);

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

            block.local_get(writer_pointer).local_set(writer_ptr);

            block.local_get(writer_ptr).call(print_i32);
            // Pack field
            field.add_pack_instructions(
                block,
                module,
                field_local,
                writer_ptr,
                calldata_reference_pointer,
                compilation_ctx,
            );

            block.local_get(writer_ptr).call(print_i32);

            block
                .i32_const(field.encoded_size(compilation_ctx) as i32)
                .local_get(writer_pointer)
                .binop(BinaryOp::I32Add)
                .local_set(writer_pointer);
        }

        block
            .local_get(calldata_reference_pointer)
            .call(print_memory_from);
    }
}
