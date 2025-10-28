use crate::translation::intermediate_types::IntermediateType;

/// This function returns true if there is a type parameter in some of the intermediate types and
/// `false` otherwise.
pub fn type_contains_generics(itype: &IntermediateType) -> bool {
    match itype {
        IntermediateType::IRef(intermediate_type)
        | IntermediateType::IMutRef(intermediate_type) => {
            type_contains_generics(intermediate_type.as_ref())
        }
        IntermediateType::ITypeParameter(_) => true,
        IntermediateType::IGenericStructInstance { types, .. } => {
            types.iter().any(type_contains_generics)
        }
        IntermediateType::IVector(inner) => type_contains_generics(inner),
        _ => false,
    }
}

/// Auxiliary functiion that recursively looks for not instantiated type parameters and
/// replaces them
pub fn replace_type_parameters(
    itype: &IntermediateType,
    instance_types: &[IntermediateType],
) -> IntermediateType {
    match itype {
        // Direct type parameter: T -> concrete_type
        IntermediateType::ITypeParameter(index) => instance_types[*index as usize].clone(),
        // Reference type parameter: &T -> &concrete_type
        IntermediateType::IRef(inner) => {
            IntermediateType::IRef(Box::new(replace_type_parameters(inner, instance_types)))
        }
        // Mutable reference type parameter: &mut T -> &mut concrete_type
        IntermediateType::IMutRef(inner) => {
            IntermediateType::IMutRef(Box::new(replace_type_parameters(inner, instance_types)))
        }
        IntermediateType::IGenericStructInstance {
            module_id,
            index,
            types,
            vm_handled_struct,
        } => IntermediateType::IGenericStructInstance {
            module_id: module_id.clone(),
            index: *index,
            types: types
                .iter()
                .map(|t| replace_type_parameters(t, instance_types))
                .collect(),
            vm_handled_struct: vm_handled_struct.clone(),
        },
        IntermediateType::IVector(inner) => {
            IntermediateType::IVector(Box::new(replace_type_parameters(inner, instance_types)))
        }
        // Non-generic type: keep as is
        _ => itype.clone(),
    }
}
