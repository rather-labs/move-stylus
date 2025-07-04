use alloy_sol_types::{SolType, sol_data};
use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{MemArg, StoreKind},
};

use crate::{CompilationContext, translation::intermediate_types::enums::IEnum};

use super::unpack_native_int::unpack_i32_type_instructions;

impl IEnum {
    pub fn add_unpack_instructions(
        _enum_: &IEnum,
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        reader_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) {
        let enum_ptr = module.locals.add(ValType::I32);

        // The enuum should occupy only 4 bytes since only the variant number is saved
        block
            .i32_const(4)
            .call(compilation_ctx.allocator)
            .local_tee(enum_ptr);

        // Read the variant number
        let encoded_size = sol_data::Uint::<8>::ENCODED_SIZE.expect("U8 should have a fixed size");
        unpack_i32_type_instructions(
            block,
            module,
            compilation_ctx.memory_id,
            reader_pointer,
            encoded_size,
        );

        block.store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        block.local_get(enum_ptr);

        // TODO: should trap if variant number is out of range
    }
}
