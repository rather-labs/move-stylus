use alloy_sol_types::{SolType, sol_data};
use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, MemArg, StoreKind},
};

use crate::{CompilationContext, translation::intermediate_types::enums::IEnum};

use super::unpack_native_int::unpack_i32_type_instructions;

impl IEnum {
    pub fn add_unpack_instructions(
        enum_: &IEnum,
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        reader_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) {
        let enum_ptr = module.locals.add(ValType::I32);
        let variant_number = module.locals.add(ValType::I32);

        let encoded_size = sol_data::Uint::<8>::ENCODED_SIZE.expect("U8 should have a fixed size");
        unpack_i32_type_instructions(
            block,
            module,
            compilation_ctx.memory_id,
            reader_pointer,
            encoded_size,
        );

        // Save the variant to check it later
        block.local_tee(variant_number);

        // Trap if the variant number is higher that the quantity of variants the enum contains
        block
            .i32_const(enum_.variants.len() as i32 - 1)
            .binop(BinaryOp::I32GtU)
            .if_else(
                None,
                |then| {
                    then.unreachable();
                },
                |_| {},
            );

        // The enum should occupy only 4 bytes since only the variant number is saved
        block
            .i32_const(4)
            .call(compilation_ctx.allocator)
            .local_tee(enum_ptr)
            .local_get(variant_number);

        // Read the variant number

        block.store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        block.local_get(enum_ptr);
    }
}
