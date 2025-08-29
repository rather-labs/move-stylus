use std::collections::HashMap;

use move_binary_format::file_format::{
    DatatypeHandleIndex, FunctionDefinition, Signature, SignatureToken, Visibility,
};
use walrus::{
    InstrSeqBuilder, LocalId, MemoryId, Module, ValType,
    ir::{LoadKind, MemArg, StoreKind},
};

use super::{
    fix_return_type,
    types_stack::{TypesStack, TypesStackError},
};

use crate::{
    CompilationContext, UserDefinedType,
    compilation_context::{ModuleData, module_data},
    generics::type_contains_generics,
    translation::{fix_call_type, intermediate_types::ISignature},
};

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

        let is_generic = signature.arguments.iter().any(type_contains_generics)
            || signature.returns.iter().any(type_contains_generics);

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

    /// Auxiliary functiion that recursively looks for not instantiated type parameters and
    /// replaces them
    fn replace_type_parameters(
        itype: &IntermediateType,
        instance_types: &[IntermediateType],
    ) -> IntermediateType {
        println!("---> {itype:?} {instance_types:?}");
        match itype {
            // Direct type parameter: T -> concrete_type
            IntermediateType::ITypeParameter(index) => instance_types[*index as usize].clone(),
            // Reference type parameter: &T -> &concrete_type
            IntermediateType::IRef(inner) => IntermediateType::IRef(Box::new(
                Self::replace_type_parameters(&inner, instance_types),
            )),
            // Mutable reference type parameter: &mut T -> &mut concrete_type
            IntermediateType::IMutRef(inner) => IntermediateType::IMutRef(Box::new(
                Self::replace_type_parameters(&inner, instance_types),
            )),
            IntermediateType::IGenericStructInstance {
                module_id,
                index,
                types,
            } => IntermediateType::IGenericStructInstance {
                module_id: module_id.clone(),
                index: *index,
                types: types
                    .iter()
                    .map(|t| Self::replace_type_parameters(t, instance_types))
                    .collect(),
            },
            IntermediateType::IExternalUserData {
                module_id,
                identifier,
                types: Some(generic_types),
            } => IntermediateType::IExternalUserData {
                module_id: module_id.clone(),
                identifier: identifier.clone(),
                types: Some(
                    generic_types
                        .iter()
                        .map(|t| Self::replace_type_parameters(t, instance_types))
                        .collect(),
                ),
            },
            IntermediateType::IVector(inner) => IntermediateType::IVector(Box::new(
                Self::replace_type_parameters(inner, instance_types),
            )),
            // Non-generic type: keep as is
            _ => itype.clone(),
        }
    }

    /// Replaces all type parameters in the function with the provided types.
    pub fn instantiate(&self, types: &[IntermediateType]) -> Self {
        let arguments = self
            .signature
            .arguments
            .iter()
            .map(|t| Self::replace_type_parameters(t, types))
            .collect();

        let returns = self
            .signature
            .returns
            .iter()
            .map(|t| Self::replace_type_parameters(t, types))
            .collect();

        let locals = self
            .locals
            .iter()
            .map(|t| Self::replace_type_parameters(t, types))
            .collect();

        let signature = ISignature { arguments, returns };
        let results = signature.get_return_wasm_types();

        let mut function_id = self.function_id.clone();
        function_id.type_instantiations = Some(types.to_vec());

        let ret = Self {
            function_id,
            signature,
            results,
            locals,
            is_generic: false,
            ..*self
        };

        println!("{ret:?}");

        ret
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

/// Looks for an IExtnernalUserData in the IntermediateType tree. If it finds it, returns it,
/// otherwise retuns None
fn look_for_external_data(itype: &IntermediateType) -> Option<&IntermediateType> {
    match itype {
        IntermediateType::IBool
        | IntermediateType::IU8
        | IntermediateType::IU16
        | IntermediateType::IU32
        | IntermediateType::IU64
        | IntermediateType::IU128
        | IntermediateType::IU256
        | IntermediateType::IAddress
        | IntermediateType::ISigner => None,
        IntermediateType::IVector(inner)
        | IntermediateType::IRef(inner)
        | IntermediateType::IMutRef(inner) => look_for_external_data(inner),
        IntermediateType::ITypeParameter(_) => None,
        IntermediateType::IStruct { module_id, index } => None,
        IntermediateType::IGenericStructInstance {
            module_id,
            index,
            types,
        } => types.iter().find(|t| look_for_external_data(*t).is_some()),
        IntermediateType::IEnum(_) => todo!(),
        IntermediateType::IExternalUserData {
            module_id,
            identifier,
            types,
        } => Some(itype),
    }
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
    function_module_data: &ModuleData,
    caller_module: &ModuleData,
) -> Result<(), TypesStackError> {
    // Verify that the types currently on the types stack correspond to the expected argument types.
    // Additionally, determine if any of these arguments are references.
    let mut has_ref = false;
    for arg in arguments.iter().rev() {
        // Here we compute the type we expect to be on the stack. The expected type can be represented
        // by two variants of the `IntermediateType` enum: `IExternalUserData` and one of the variants
        // corresponding to generic or concrete structs/enums.
        //
        // `IExternalUserData` and those variants have a one-to-one correspondence: one is used from the
        // perspective of a module that did not define the datatype, and the other is used from the
        // perspective of the module that defined it.
        //
        // There are cases where we encounter an `IExternalUserData` but the function expects an
        // `IGenericStructInstance` (or another struct/enum variant), and vice versa.
        //
        // This happens when the caller of the function and the callee are in different modules...
        let stack_type = if caller_module.id != function_module_data.id {
            let type_ = types_stack.pop()?;

            match &look_for_external_data(&type_) {
                // If we find an `IExternalUserData` at the top of the stack (or inside another type) and the
                // `IExternalUserData`’s module matches the callee’s module, then the callee will expect an
                // `IntermediateType` defined within its own module (for example, an `IGenericStructInstance`
                // that corresponds to the encountered `IExternalUserData`).
                //
                // In this case, we simply return `arg`, since it already matches what the callee expects.
                Some(IntermediateType::IExternalUserData { module_id, .. })
                    if *module_id == function_module_data.id =>
                {
                    arg.clone()
                }
                // Otherwise, if the data type’s module does not match the callee’s module, we
                // return it directly.
                _ => type_,
            }
        }
        // If the caller’s module and the callee’s module do not match, but we encounter an
        // `IExternalUserData` that belongs to the callee’s module, the callee will expect an
        // `IntermediateType` defined within its own module (for example, an `IGenericStructInstance`
        // corresponding to the `IExternalUserData`).
        //
        // In this case, the `fix_call_type` function returns the expected `IntermediateType`.
        else {
            fix_call_type(&types_stack.pop()?, compilation_ctx, function_module_data)
        };

        assert_eq!(&stack_type, arg);

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
