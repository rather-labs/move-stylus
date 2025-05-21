use anyhow::Result;
use move_binary_format::file_format::{CodeUnit, FunctionDefinition, Signature};
use walrus::{
    ir::{LoadKind, MemArg, StoreKind},
    InstrSeqBuilder, LocalId, MemoryId, Module, ValType,
};

use crate::{translation::intermediate_types::ISignature, CompilationContext};

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

impl<'a> MappedFunction<'a> {
    pub fn new(
        name: String,
        move_arguments: &Signature,
        move_returns: &Signature,
        move_definition: FunctionDefinition,
        module: &mut Module,
        move_module_signatures: &'a [Signature],
    ) -> Self {
        assert!(
            move_definition.acquires_global_resources.is_empty(),
            "Acquiring global resources is not supported yet"
        );

        let code = move_definition.code.clone().expect("Function has no code");

        let signature = ISignature::from_signatures(move_arguments, move_returns);
        let function_arguments = signature.get_argument_wasm_types();
        let function_returns = signature.get_return_wasm_types();

        assert!(
            function_returns.len() <= 1,
            "Multiple return values is not enabled in Stylus VM"
        );

        // === Handle argument locals ===
        let arg_local_ids = function_arguments
            .iter()
            .map(|arg| module.locals.add(*arg))
            .collect::<Vec<LocalId>>();

        let arg_intermediate_types = move_arguments.0.iter().map(IntermediateType::try_from);

        // === Handle declared locals ===
        let move_locals = &code.locals;
        let signature_tokens = &move_module_signatures[move_locals.0 as usize].0;

        let local_intermediate_types = signature_tokens.iter().map(IntermediateType::try_from);

        let local_ids = local_intermediate_types
            .clone()
            .flat_map(|token| token.map(ValType::from))
            .map(|valty| module.locals.add(valty));

        // === Combine all locals and types ===
        let local_variables = arg_local_ids.clone().into_iter().chain(local_ids).collect();

        let local_variables_type = arg_intermediate_types
            .chain(local_intermediate_types)
            .collect::<Result<Vec<IntermediateType>, anyhow::Error>>()
            // TODO: unwrap
            .unwrap();

        Self {
            name,
            signature,
            move_definition,
            move_code_unit: code,
            local_variables,
            local_variables_type,
            arg_local_ids,
            move_module_signatures,
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
