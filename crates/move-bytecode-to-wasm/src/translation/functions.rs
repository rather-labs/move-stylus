use std::collections::HashMap;

use move_binary_format::file_format::{
    DatatypeHandleIndex, FunctionDefinition, Signature, SignatureToken, Visibility,
};
use walrus::{
    InstrSeqBuilder, LocalId, MemoryId, Module, ValType,
    ir::{LoadKind, MemArg, StoreKind},
};

use super::types_stack::{TypesStack, TypesStackError};

use crate::{CompilationContext, UserDefinedType, translation::intermediate_types::ISignature};

use super::{intermediate_types::IntermediateType, table::FunctionId};

#[derive(Debug)]
pub struct MappedFunction {
    pub function_id: FunctionId,
    pub signature: ISignature,
    pub locals: Vec<IntermediateType>,
    pub results: Vec<ValType>,

    /// Flag that tells us if the function can be used as an entrypoint
    pub is_entry: bool,

    /// Flag that tells us if the function is a native function
    pub is_native: bool,

    /// Flag that tells us if the function contains generic arguments or return values
    pub is_generic: bool,
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

        // Declared locals
        let locals = move_locals
            .iter()
            .map(|s| IntermediateType::try_from_signature_token(s, handles_map))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let is_generic = signature.arguments.iter().any(|a| match a {
            IntermediateType::IRef(intermediate_type)
            | IntermediateType::IMutRef(intermediate_type) => {
                matches!(
                    intermediate_type.as_ref(),
                    IntermediateType::ITypeParameter(_)
                )
            }
            IntermediateType::ITypeParameter(_) => true,
            _ => false,
        }) || signature.returns.iter().any(|a| match a {
            IntermediateType::IRef(intermediate_type)
            | IntermediateType::IMutRef(intermediate_type) => {
                matches!(
                    intermediate_type.as_ref(),
                    IntermediateType::ITypeParameter(_)
                )
            }
            IntermediateType::ITypeParameter(_) => true,
            _ => false,
        });

        Self {
            function_id,
            signature,
            locals,
            results,
            // TODO: change to function_definition.is_entry
            is_entry: function_definition.visibility == Visibility::Public,
            is_native: function_definition.is_native(),
            is_generic,
        }
    }
}

impl MappedFunction {
    pub fn get_local_ir(&self, local_index: usize) -> &IntermediateType {
        if local_index < self.signature.arguments.len() {
            &self.signature.arguments[local_index]
        } else {
            &self.locals[local_index - self.signature.arguments.len()]
        }
    }

    /// Replaces all type parameters in the function with the provided types.
    pub fn instantiate(&self, types: &[IntermediateType]) -> Self {
        println!("2 {types:?}");
        // Helper function to instantiate a single type
        let instantiate_type = |t: &IntermediateType| -> IntermediateType {
            println!("itype: {t:?}");
            match t {
                // Direct type parameter: T -> concrete_type
                IntermediateType::ITypeParameter(index) => types[*index as usize].clone(),
                // Reference type parameter: &T -> &concrete_type
                IntermediateType::IRef(inner) => {
                    if let IntermediateType::ITypeParameter(index) = inner.as_ref() {
                        let concrete_type = types[*index as usize].clone();

                        // If the concrete type is already a reference, return it as is
                        // Otherwise, wrap it in a reference
                        if let IntermediateType::IRef(_) = &concrete_type {
                            concrete_type
                        } else {
                            IntermediateType::IRef(Box::new(concrete_type))
                        }
                    } else {
                        t.clone()
                    }
                }
                // Mutable reference type parameter: &mut T -> &mut concrete_type
                IntermediateType::IMutRef(inner) => {
                    if let IntermediateType::ITypeParameter(index) = inner.as_ref() {
                        let concrete_type = types[*index as usize].clone();
                        if let IntermediateType::IMutRef(_) = &concrete_type {
                            concrete_type
                        } else {
                            IntermediateType::IMutRef(Box::new(concrete_type))
                        }
                    } else {
                        t.clone()
                    }
                }
                IntermediateType::IGenericStructInstance {
                    module_id,
                    index,
                    types: struct_types,
                } => {
                    println!("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
                    IntermediateType::IGenericStructInstance {
                        module_id: module_id.clone(),
                        index: *index,
                        types: struct_types
                            .iter()
                            .map(|t| {
                                if let IntermediateType::ITypeParameter(index) = t {
                                    types[*index as usize].clone()
                                } else {
                                    t.clone()
                                }
                            })
                            .collect(),
                    }
                }
                // Non-generic type: keep as is
                _ => t.clone(),
            }
        };

        let arguments = self
            .signature
            .arguments
            .iter()
            .map(instantiate_type)
            .collect();

        println!("3 {:?}", self.signature.returns);

        let returns = self
            .signature
            .returns
            .iter()
            .map(instantiate_type)
            .collect();

        println!("4 {returns:?}");

        let locals = self.locals.iter().map(instantiate_type).collect();

        let signature = ISignature { arguments, returns };
        let results = signature.get_return_wasm_types();

        let mut function_id = self.function_id.clone();
        function_id.type_instantiations = Some(types.to_vec());

        Self {
            function_id,
            signature,
            results,
            locals,
            is_generic: false,
            ..*self
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

/// This function sets up the arguments for a function call.
///
/// It processes each argument type, checking if it is an immutable (`IRef`) or mutable (`IMutRef`) reference.
/// If a reference is detected, the function ensures that the pointer to the referenced data is loaded.
pub fn prepare_function_arguments(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    arguments: &[IntermediateType],
    compilation_ctx: &CompilationContext,
    types_stack: &mut TypesStack,
) -> Result<(), TypesStackError> {
    // Verify that the types currently on the types stack correspond to the expected argument types.
    // Additionally, determine if any of these arguments are references.
    let mut has_ref = false;
    for arg in arguments.iter().rev() {
        types_stack.pop_expecting(arg)?;
        has_ref = has_ref
            || matches!(
                arg,
                IntermediateType::IRef(_) | IntermediateType::IMutRef(_)
            );
    }

    // If the function has any reference arguments, we need to load the Ref pointer before calling the function
    if has_ref {
        // i. Spill all args from the value stack into locals (last arg first)
        let mut spilled: Vec<LocalId> = Vec::new();

        for arg_ty in arguments.iter().rev() {
            if matches!(
                arg_ty,
                IntermediateType::IRef(_) | IntermediateType::IMutRef(_)
            ) {
                // If the argument is a Ref, load the pointer
                builder.load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            }
            spilled.push(arg_ty.add_stack_to_local_instructions(module, builder));
        }

        // ii. Rebuild the operand stack in call order (first .. last)
        for loc in spilled.into_iter().rev() {
            builder.local_get(loc);
        }
    };

    Ok(())
}
