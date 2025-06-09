use walrus::{
    FunctionBuilder, FunctionId, InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use super::RuntimeFunction;
use crate::CompilationContext;
use crate::translation::intermediate_types::IntermediateType;
use crate::wasm_builder_extensions::WasmBuilderExtension;

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

    let vec_ref = module.locals.add(ValType::I32);
    let idx1_i64 = module.locals.add(ValType::I64);
    let idx2_i64 = module.locals.add(ValType::I64);

    let idx2 = module.locals.add(ValType::I32);
    let idx1 = module.locals.add(ValType::I32);
    let ptr = module.locals.add(ValType::I32);
    let len = module.locals.add(ValType::I32);

    let size = inner.stack_data_size() as i32;

    // Load vector ptr and len
    builder
        .local_get(vec_ref)
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

        let downcast_f = RuntimeFunction::DowncastU64ToU32.get(module, None, None);

        block.local_get(idx1_i64).call(downcast_f).local_set(idx1);
        block.local_get(idx2_i64).call(downcast_f).local_set(idx2);

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
    function.finish(vec![vec_ref, idx1_i64, idx2_i64], &mut module.funcs)
}
