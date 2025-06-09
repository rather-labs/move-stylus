use walrus::{
    FunctionBuilder, FunctionId, InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use super::RuntimeFunction;
use crate::CompilationContext;
use crate::translation::intermediate_types::IntermediateType;
use crate::wasm_builder_extensions::WasmBuilderExtension;

/// Swaps the elements at two indices in the vector. Abort the execution if any of the indice
/// is out of bounds.
///
/// ```..., vector_reference, u64_value(1), u64_value(2) -> ...```
pub fn vec_swap_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    inner: &IntermediateType,
) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I64, ValType::I64],
        &[],
    );
    let mut builder = function
        .name(RuntimeFunction::VecSwap.name().to_owned())
        .func_body();

    let ptr = module.locals.add(ValType::I32);
    let idx1_i64 = module.locals.add(ValType::I64);
    let idx2_i64 = module.locals.add(ValType::I64);

    let idx2 = module.locals.add(ValType::I32);
    let idx1 = module.locals.add(ValType::I32);
    let len = module.locals.add(ValType::I32);

    let downcast_f = RuntimeFunction::DowncastU64ToU32.get(module, None, None);

    builder.local_get(idx1_i64).call(downcast_f).local_set(idx1);
    builder.local_get(idx2_i64).call(downcast_f).local_set(idx2);

    let size = inner.stack_data_size() as i32;

    // Load vector ptr and len
    builder
        .local_get(ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_tee(ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(len);

    builder.block(None, |block| {
        let block_id = block.id();

        block
            .local_get(idx1_i64)
            .local_get(idx2_i64)
            .binop(BinaryOp::I64Eq)
            .br_if(block_id);

        // Helper: emit trap if idx >= len
        let trap_if_idx_oob = |b: &mut InstrSeqBuilder, idx: LocalId| {
            b.local_get(idx)
                .local_get(len)
                .binop(BinaryOp::I32GeU)
                .if_else(
                    None,
                    |then_| {
                        then_.unreachable();
                    },
                    |_| {},
                );
        };

        trap_if_idx_oob(block, idx1);
        trap_if_idx_oob(block, idx2);

        let (valtype, storekind, loadkind) = match inner {
            IntermediateType::IU64 => (
                ValType::I64,
                StoreKind::I64 { atomic: false },
                LoadKind::I64 { atomic: false },
            ),
            _ => (
                ValType::I32,
                StoreKind::I32 { atomic: false },
                LoadKind::I32 { atomic: false },
            ),
        };

        // Swap elements
        let aux = module.locals.add(valtype);

        let ptr1 = module.locals.add(ValType::I32);
        let ptr2 = module.locals.add(ValType::I32);

        block.vec_ptr_at(ptr, idx1, size);
        block.local_set(ptr1);

        block.vec_ptr_at(ptr, idx2, size);
        block.local_set(ptr2);

        // Load elem 1 into aux
        block
            .local_get(ptr1)
            .load(
                compilation_ctx.memory_id,
                loadkind,
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )
            .local_set(aux);

        // Store elem 2 into ptr1
        block
            .local_get(ptr1)
            .local_get(ptr2)
            .load(
                compilation_ctx.memory_id,
                loadkind,
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )
            .store(
                compilation_ctx.memory_id,
                storekind,
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

        // Store elem 1 into ptr2
        block.local_get(ptr2).local_get(aux).store(
            compilation_ctx.memory_id,
            storekind,
            MemArg {
                align: 0,
                offset: 0,
            },
        );
    });
    function.finish(vec![ptr, idx1_i64, idx2_i64], &mut module.funcs)
}

/// Pop an element from the end of vector. Aborts if the vector is empty.
///
/// Stack transition:
///
/// ```..., vector_reference -> ..., element```
pub fn vec_pop_back_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    inner: &IntermediateType,
) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32],
        &[match inner {
            IntermediateType::IU64 => ValType::I64,
            _ => ValType::I32,
        }],
    );
    let mut builder = function
        .name(RuntimeFunction::VecPopBack.name().to_owned())
        .func_body();

    let size = inner.stack_data_size() as i32;
    let ptr = module.locals.add(ValType::I32);
    let len = module.locals.add(ValType::I32);

    builder
        .local_get(ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_tee(ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(len);

    // Trap if vector length == 0
    builder
        .local_get(len)
        .i32_const(0)
        .binop(BinaryOp::I32Eq)
        .if_else(
            None,
            |then| {
                then.unreachable(); // cannot pop from empty vector
            },
            |_| {},
        );

    // Vector length is reduced by 1
    builder
        .local_get(ptr)
        .local_get(len)
        .i32_const(1)
        .binop(BinaryOp::I32Sub)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

    // Update vector length
    builder
        .local_get(ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(len);

    builder.vec_ptr_at(ptr, len, size);

    match inner {
        IntermediateType::IBool
        | IntermediateType::IU8
        | IntermediateType::IU16
        | IntermediateType::IU32
        | IntermediateType::IU128
        | IntermediateType::IU256
        | IntermediateType::IAddress
        | IntermediateType::ISigner
        | IntermediateType::IVector(_) => {
            builder.load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );
        }
        IntermediateType::IU64 => {
            builder.load(
                compilation_ctx.memory_id,
                LoadKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );
        }
        IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
            panic!("VecPopBack operation is not allowed on reference types");
        }
    }

    function.finish(vec![ptr], &mut module.funcs)
}
