use std::collections::HashMap;

use move_binary_format::file_format::{
    DatatypeHandleIndex, FunctionDefinition, Signature, SignatureToken,
};
use walrus::{
    InstrSeqBuilder, LocalId, MemoryId, Module, ValType,
    ir::{LoadKind, MemArg, StoreKind},
};

use crate::{
    CompilationContext, UserDefinedType, compilation_context::UserDefinedGenericType,
    translation::intermediate_types::ISignature,
};

use super::intermediate_types::IntermediateType;

pub struct MappedFunction {
    pub name: String,
    pub signature: ISignature,
    pub function_definition: FunctionDefinition,
    pub function_locals: Vec<LocalId>,
    pub function_locals_ir: Vec<IntermediateType>,
    pub arg_locals: Vec<LocalId>,
}

impl MappedFunction {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        move_args: &Signature,
        move_rets: &Signature,
        move_locals: &Signature,
        move_def: FunctionDefinition,
        handles_map: &HashMap<DatatypeHandleIndex, UserDefinedType>,
        handles_generics_instances_map: &HashMap<
            (DatatypeHandleIndex, Vec<SignatureToken>),
            UserDefinedGenericType,
        >,
        module: &mut Module,
    ) -> Self {
        assert!(
            move_def.acquires_global_resources.is_empty(),
            "Acquiring global resources is not supported yet"
        );

        let signature = ISignature::from_signatures(
            move_args,
            move_rets,
            handles_map,
            handles_generics_instances_map,
        );
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

        let ir_arg_types = move_args.0.iter().map(|s| {
            IntermediateType::try_from_signature_token(
                s,
                handles_map,
                handles_generics_instances_map,
            )
        });

        // Declared locals
        let ir_declared_locals_types = move_locals.0.iter().map(|s| {
            IntermediateType::try_from_signature_token(
                s,
                handles_map,
                handles_generics_instances_map,
            )
        });

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
            function_definition: move_def,
            function_locals: local_variables,
            function_locals_ir: local_variables_type,
            arg_locals: wasm_arg_locals,
        }
    }

    /// Converts value-based function arguments into heap-allocated pointers.
    ///
    /// For each value-type argument (like u64, u32, etc.), this stores the value in linear memory
    /// and updates the local to hold a pointer to that memory instead. This allows treating all
    /// arguments as pointers in later code.
    pub fn box_args(
        &mut self,
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
    ) {
        // Store the changes we need to make
        let mut updates = Vec::new();

        // Iterate over the mapped function arguments
        for (local, ty) in self
            .function_locals
            .iter()
            .zip(self.signature.arguments.iter())
        {
            builder.local_get(*local);
            match ty {
                IntermediateType::IU64 => {
                    let outer_ptr = module.locals.add(ValType::I32);
                    ty.box_local_instructions(module, builder, compilation_ctx, outer_ptr);

                    if let Some(index) = self.function_locals.iter().position(|&id| id == *local) {
                        updates.push((index, outer_ptr));
                    } else {
                        panic!(
                            "Couldn't find original local {:?} in mapped_function",
                            local
                        );
                    }
                }
                _ => {
                    ty.box_local_instructions(module, builder, compilation_ctx, *local);
                }
            }
        }

        for (index, pointer) in updates {
            self.function_locals[index] = pointer;
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
