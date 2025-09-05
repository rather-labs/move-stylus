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

/// Extracts the `ITypeParameters` contained in `itype`. The extracted `ITypeParameters` will then
/// be replaced with concrete types
pub fn extract_generic_type_parameters(itype: &IntermediateType) -> Vec<IntermediateType> {
    match itype {
        IntermediateType::IRef(intermediate_type)
        | IntermediateType::IMutRef(intermediate_type) => {
            extract_generic_type_parameters(intermediate_type.as_ref())
        }
        IntermediateType::ITypeParameter(_) => vec![itype.clone()],
        IntermediateType::IGenericStructInstance { types, .. } => {
            types.iter().fold(vec![], |mut acc, t| {
                acc.append(&mut extract_generic_type_parameters(t));
                acc
            })
        }
        IntermediateType::IVector(inner) => extract_generic_type_parameters(inner),
        _ => vec![],
    }
}

/// This function extracts the instance type so we can pass it to the instantiation functions. For
/// example:
///
/// If we have:
///
/// `generic_type` = `IVector(ITypeParameter(0))`
/// `instantiated_type` = `IVector(IU64)`
///
/// this function will return `IU64`, since the instantiation needs to replace `ITypeParameter(0)`
/// with `IU64`.
///
/// If we have:
///
/// `generic_type` = `IVector(ITypeParameter(0))`
/// `instantiated_type` = `IVector(IVector(IU64))`
///
/// this function will return `IVector(IU64)`, since the instantiation needs to replace
/// `ITypeParameter(0)` with `IVector(IU64)`.
///
/// The index corresponds to which generic type paramneter we are extracting
pub fn extract_type_instances_from_stack(
    generic_type: &IntermediateType,
    instantiated_type: &IntermediateType,
) -> Option<IntermediateType> {
    match generic_type {
        IntermediateType::ITypeParameter(_) => match instantiated_type {
            IntermediateType::IRef(instantiated_inner)
            | IntermediateType::IMutRef(instantiated_inner) => {
                extract_type_instances_from_stack(generic_type, instantiated_inner)
            }

            _ => Some(instantiated_type.clone()),
        },
        IntermediateType::IVector(inner) => {
            if let IntermediateType::IVector(instantiated_inner) = instantiated_type {
                extract_type_instances_from_stack(inner, instantiated_inner)
            } else {
                None
            }
        }
        IntermediateType::IRef(inner) => {
            if let IntermediateType::IRef(instantiated_inner) = instantiated_type {
                extract_type_instances_from_stack(inner, instantiated_inner)
            } else {
                None
            }
        }
        IntermediateType::IMutRef(inner) => {
            if let IntermediateType::IMutRef(instantiated_inner) = instantiated_type {
                extract_type_instances_from_stack(inner, instantiated_inner)
            } else {
                None
            }
        }
        IntermediateType::IGenericStructInstance {
            types: generic_types,
            ..
        } => {
            if let IntermediateType::IGenericStructInstance {
                types: instantaited_types,
                ..
            } = instantiated_type
            {
                for (gt, it) in generic_types.iter().zip(instantaited_types) {
                    let res = extract_type_instances_from_stack(gt, it);
                    if res.is_some() {
                        return res;
                    }
                }
                None
            } else {
                None
            }
        }
        _ => None,
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
        } => IntermediateType::IGenericStructInstance {
            module_id: module_id.clone(),
            index: *index,
            types: types
                .iter()
                .map(|t| replace_type_parameters(t, instance_types))
                .collect(),
        },
        IntermediateType::IVector(inner) => {
            IntermediateType::IVector(Box::new(replace_type_parameters(inner, instance_types)))
        }
        // Non-generic type: keep as is
        _ => itype.clone(),
    }
}

/// This function is used to instantiate generic types that may appear in the `inner` type of a
/// vector
pub fn instantiate_vec_type_parameters(
    inner: &IntermediateType,
    function_type_instaces: &[IntermediateType],
) -> IntermediateType {
    let generic_types = extract_generic_type_parameters(inner);
    let concrete_types = generic_types
        .iter()
        .map(|g| {
            if let IntermediateType::ITypeParameter(i) = g {
                function_type_instaces[*i as usize].clone()
            } else {
                g.clone()
            }
        })
        .collect::<Vec<IntermediateType>>();

    replace_type_parameters(inner, &concrete_types)
}
