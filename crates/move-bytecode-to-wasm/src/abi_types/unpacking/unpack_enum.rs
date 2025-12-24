use alloy_sol_types::{SolType, sol_data};
use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, MemArg, StoreKind},
};

use crate::{
    CompilationContext, abi_types::error::AbiError, runtime::RuntimeFunction,
    translation::intermediate_types::enums::IEnum,
};

impl IEnum {
    pub fn add_unpack_instructions(
        enum_: &IEnum,
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        reader_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) -> Result<(), AbiError> {
        let enum_ptr = module.locals.add(ValType::I32);
        let variant_number = module.locals.add(ValType::I32);

        let encoded_size =
            sol_data::Uint::<8>::ENCODED_SIZE.ok_or(AbiError::UnableToGetTypeAbiSize)?;
        let unpack_u32_function = RuntimeFunction::UnpackU32.get(module, Some(compilation_ctx))?;
        block
            .local_get(reader_pointer)
            .i32_const(encoded_size as i32)
            .call(unpack_u32_function);

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
            .local_get(variant_number)
            .store(
                compilation_ctx.memory_id,
                StoreKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

        block.local_get(enum_ptr);

        Ok(())
    }
}
