use walrus::{InstrSeqBuilder, LocalId, Module, ValType, ir::BinaryOp};

use crate::{CompilationContext, translation::intermediate_types::structs::IStruct};

use super::Packable;

impl IStruct {
    pub fn add_pack_instructions(
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        local: LocalId,
        writer_pointer: LocalId,
        calldata_reference_pointer: LocalId,
        compilation_ctx: &CompilationContext,
        struct_index: usize,
    ) {
        let struct_to_pack = compilation_ctx
            .module_structs
            .get(struct_index)
            .unwrap_or_else(|| panic!("packing struct: struct with index {struct_index} not found in compilation context"));

        // Packing an struct is simply packing every inner value one besides the other
        for pack_type in struct_to_pack.fields.values() {
            // for pack_type in struct_to_pack.fields.iter().rev() {
            let local = if pack_type.stack_data_size() == 8 {
                module.locals.add(ValType::I64)
            } else {
                module.locals.add(ValType::I32)
            };

            pack_type.add_pack_instructions(
                builder,
                module,
                local,
                writer_pointer,
                calldata_reference_pointer,
                compilation_ctx,
            );

            builder
                .i32_const(pack_type.stack_data_size() as i32)
                .local_get(writer_pointer)
                .binop(BinaryOp::I32Add)
                .local_set(writer_pointer);
        }
    }
}
