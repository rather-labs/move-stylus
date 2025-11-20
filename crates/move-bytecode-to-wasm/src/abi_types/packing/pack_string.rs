use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use super::pack_native_int::pack_i32_type_instructions;
use crate::{
    CompilationContext, abi_types::error::AbiError,
    translation::intermediate_types::IntermediateType, vm_handled_types::string::String_,
};

impl String_ {
    pub fn add_pack_instructions(
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        local: LocalId,
        writer_pointer: LocalId,
        calldata_reference_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) -> Result<(), AbiError> {
        let data_pointer = module.locals.add(ValType::I32);
        let inner_data_reference = module.locals.add(ValType::I32);

        // String in move have the following form:
        // public struct String has copy, drop, store {
        //   bytes: vector<u8>,
        // }
        //
        // So we need to perform a load first to get to the inner vector
        builder
            .local_get(local)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )
            .local_set(local);

        let len = IntermediateType::IU32.add_load_memory_to_local_instructions(
            module,
            builder,
            local,
            compilation_ctx.memory_id,
        )?;

        // Allocate space for the text, padding by 32 bytes plus 32 bytes for the length
        builder
            .local_get(len)
            .i32_const(31)
            .binop(BinaryOp::I32Add)
            .i32_const(!31)
            .binop(BinaryOp::I32And)
            .i32_const(32)
            .binop(BinaryOp::I32Add)
            .call(compilation_ctx.allocator)
            .local_tee(data_pointer);

        // The value stored at this param position should be the distance from the start of this
        // calldata portion to the pointer
        let reference_value = module.locals.add(ValType::I32);

        builder
            .local_get(calldata_reference_pointer)
            .binop(BinaryOp::I32Sub)
            .local_set(reference_value);

        pack_i32_type_instructions(
            builder,
            module,
            compilation_ctx.memory_id,
            reference_value,
            writer_pointer,
        )?;

        // Set the local to point to the first element
        builder
            .local_get(local)
            .i32_const(8)
            .binop(BinaryOp::I32Add)
            .local_set(local);

        /*
         *  Store the values at allocated memory at the end of calldata
         */

        // Length
        pack_i32_type_instructions(
            builder,
            module,
            compilation_ctx.memory_id,
            len,
            data_pointer,
        )?;

        // increment the data pointer
        builder
            .local_get(data_pointer)
            .i32_const(32)
            .binop(BinaryOp::I32Add)
            .local_tee(data_pointer)
            .local_set(inner_data_reference); // This will be the reference for next allocated calldata

        // Outer block: if the vector length is 0, we skip to the end
        builder.block(None, |outer_block| {
            let outer_block_id = outer_block.id();

            // Check if length == 0
            outer_block
                .local_get(len)
                .i32_const(0)
                .binop(BinaryOp::I32Eq)
                .br_if(outer_block_id);

            // Loop through the vector values
            let i = module.locals.add(ValType::I32);
            outer_block.i32_const(0).local_set(i);
            outer_block.loop_(None, |loop_block| {
                let loop_block_id = loop_block.id();

                loop_block
                    .local_get(data_pointer)
                    .local_get(local)
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

                // increment the local to point to next first value
                loop_block
                    .local_get(local)
                    .i32_const(4)
                    .binop(BinaryOp::I32Add)
                    .local_set(local);

                // increment data pointer
                loop_block
                    .local_get(data_pointer)
                    .i32_const(1)
                    .binop(BinaryOp::I32Add)
                    .local_set(data_pointer);

                // increment i
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
        });

        Ok(())
    }
}
