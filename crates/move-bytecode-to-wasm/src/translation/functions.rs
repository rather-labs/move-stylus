use anyhow::Result;
use move_binary_format::file_format::{CodeUnit, FunctionDefinition, Signature};
use walrus::{
    InstrSeqBuilder, LocalId, MemoryId, Module, ValType,
    ir::{LoadKind, MemArg, StoreKind},
};

use crate::{CompilationContext, translation::intermediate_types::ISignature};

use super::intermediate_types::IntermediateType;

pub struct MappedFunction<'a> {
    pub name: String,
    pub signature: ISignature,
    pub move_definition: FunctionDefinition,
    pub move_code_unit: CodeUnit,
    pub local_variables: Vec<LocalId>,
    pub local_variables_type: Vec<IntermediateType>,
    pub arg_local_ids: Vec<LocalId>,
    pub move_module_signatures: &'a [Signature],
}

//@ Do we need to pass the whole move_module_signatures to each mapped function?
impl<'a> MappedFunction<'a> {
    pub fn new(
        name: String,
        move_args: &Signature,
        move_rets: &Signature,
        move_def: FunctionDefinition,
        module: &mut Module,
        sig_pool: &'a [Signature],
    ) -> Self {
        assert!(
            move_def.acquires_global_resources.is_empty(),
            "Acquiring global resources is not supported yet"
        );

        let code = move_def.code.clone().expect("Function has no code");

        let signature = ISignature::from_signatures(move_args, move_rets);
        let wasm_arg_types = signature.get_argument_wasm_types();
        let wasm_ret_types = signature.get_return_wasm_types();

        assert!(
            wasm_ret_types.len() <= 1,
            "Multiple return values not supported"
        );

        // WASM locals for arguments
        let wasm_arg_locals: Vec<LocalId> = wasm_arg_types
            .iter()
            .map(|ty| module.locals.add(*ty))
            .collect();

        let ir_arg_types = move_args.0.iter().map(IntermediateType::try_from);

        // Declared locals
        let ir_declared_locals_types = sig_pool[code.locals.0 as usize]
            .0
            .iter()
            .map(IntermediateType::try_from);

        let wasm_declared_locals = ir_declared_locals_types
            .clone()
            .map(|ty| match ty {
                Ok(IntermediateType::IU64) => ValType::I32, // to store pointer instead of i64
                Ok(ref other) => ValType::from(other),
                Err(_) => ValType::I32,
            })
            .map(|val| module.locals.add(val));

        // Combine all
        let local_variables = wasm_arg_locals
            .clone()
            .into_iter()
            .chain(wasm_declared_locals)
            .collect();

        let local_variables_type = ir_arg_types
            .chain(ir_declared_locals_types)
            .collect::<Result<Vec<_>, _>>()
            .expect("Failed to parse types");

        Self {
            name,
            signature,
            move_definition: move_def,
            move_code_unit: code,
            local_variables,
            local_variables_type,
            arg_local_ids: wasm_arg_locals,
            move_module_signatures: sig_pool,
        }
    }
}

/// Adds the instructions to unpack the return values from memory
///
/// The returns values are read from memory and pushed to the stack
pub fn add_unpack_function_return_values_instructions(
    builder: &mut InstrSeqBuilder,
    module: &mut Module,
    returns: &[IntermediateType],
    memory: MemoryId,
) {
    if returns.is_empty() {
        return;
    }

    let pointer = module.locals.add(ValType::I32);
    builder.local_set(pointer);

    let mut offset = 0;
    for return_ty in returns.iter() {
        builder.local_get(pointer);
        if return_ty.stack_data_size() == 4 {
            builder.load(
                memory,
                LoadKind::I32 { atomic: false },
                MemArg { align: 0, offset },
            );
        } else if return_ty.stack_data_size() == 8 {
            builder.load(
                memory,
                LoadKind::I64 { atomic: false },
                MemArg { align: 0, offset },
            );
        } else {
            unreachable!("Unsupported type size");
        }
        offset += return_ty.stack_data_size();
    }
}

/// Packs the return values into a tuple if the function has return values
///
/// This is necessary because the Stylus VM does not support multiple return values
/// Values are written to memory and a pointer to the first value is returned
pub fn prepare_function_return(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    returns: &[IntermediateType],
    compilation_ctx: &CompilationContext,
) {
    if !returns.is_empty() {
        let mut locals = Vec::new();
        let mut total_size = 0;
        for return_ty in returns.iter().rev() {
            let local = return_ty.add_stack_to_local_instructions(module, builder);
            locals.push(local);
            total_size += return_ty.stack_data_size();
        }
        locals.reverse();

        let pointer = module.locals.add(ValType::I32);

        builder.i32_const(total_size as i32);
        builder.call(compilation_ctx.allocator);
        builder.local_set(pointer);

        let mut offset = 0;
        for (return_ty, local) in returns.iter().zip(locals.iter()) {
            builder.local_get(pointer);
            builder.local_get(*local);
            if return_ty.stack_data_size() == 4 {
                builder.store(
                    compilation_ctx.memory_id,
                    StoreKind::I32 { atomic: false },
                    MemArg { align: 0, offset },
                );
            } else if return_ty.stack_data_size() == 8 {
                builder.store(
                    compilation_ctx.memory_id,
                    StoreKind::I64 { atomic: false },
                    MemArg { align: 0, offset },
                );
            } else {
                unreachable!("Unsupported type size");
            }
            offset += return_ty.stack_data_size();
        }

        builder.local_get(pointer);
    }

    builder.return_();
}
