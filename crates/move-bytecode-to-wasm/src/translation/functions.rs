use std::collections::HashMap;

use move_binary_format::file_format::{
    DatatypeHandleIndex, FunctionDefinition, Signature, SignatureToken, Visibility,
};
use walrus::{
    InstrSeqBuilder, MemoryId, Module, ValType,
    ir::{LoadKind, MemArg, StoreKind},
};

use crate::{CompilationContext, UserDefinedType, translation::intermediate_types::ISignature};

use super::{intermediate_types::IntermediateType, table::FunctionId};

#[derive(Debug)]
pub struct MappedFunction {
    pub function_id: FunctionId,
    pub signature: ISignature,
    pub locals: Vec<IntermediateType>,
    pub arguments: Vec<IntermediateType>,
    pub results: Vec<ValType>,

    /// Flag that tells us if the function can be used as an entrypoint
    pub is_entry: bool,

    /// Flag that tells us if the function is a native function
    pub is_native: bool,
}

impl MappedFunction {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        function_id: FunctionId,
        move_args: &Signature,
        move_rets: &Signature,
        move_locals: &[SignatureToken],
        function_definition: &FunctionDefinition,
        handles_map: &HashMap<DatatypeHandleIndex, UserDefinedType>,
    ) -> Self {
        let signature = ISignature::from_signatures(move_args, move_rets, handles_map);
        let results = signature.get_return_wasm_types();

        assert!(results.len() <= 1, "Multiple return values not supported");

        let arguments = move_args
            .0
            .iter()
            .map(|s| IntermediateType::try_from_signature_token(s, handles_map))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        // Declared locals
        let locals = move_locals
            .iter()
            .map(|s| IntermediateType::try_from_signature_token(s, handles_map))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        Self {
            function_id,
            signature,
            locals,
            arguments,
            results,
            // TODO: change to function_definition.is_entry
            is_entry: function_definition.visibility == Visibility::Public,
            is_native: function_definition.is_native(),
        }
    }
}

impl MappedFunction {
    pub fn get_local_ir(&self, local_index: usize) -> &IntermediateType {
        if local_index < self.arguments.len() {
            &self.arguments[local_index]
        } else {
            &self.locals[local_index - self.arguments.len()]
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
