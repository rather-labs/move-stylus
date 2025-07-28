use crate::UserDefinedType;
use crate::abi_types::vm_handled_datatypes::TxContext;
use crate::translation::intermediate_types::IntermediateType;
use crate::translation::table::FunctionId;
use move_binary_format::file_format::{
    Ability, AbilitySet, CompiledModule, DatatypeHandleIndex, FunctionDefinition, Signature,
    SignatureToken, Visibility,
};
use std::collections::HashMap;

const INIT_FUNCTION_NAME: &str = "init";

// The init() function is a special function that is called once when the module is first deployed,
// so it is a good place to put the code that initializes module's objects and sets up the environment and configuration.
//
// For the init() function to be considered valid, it must adhere to the following requirements:
// 1. It must be named `init`.
// 2. It must be private.
// 3. It must have &TxContext or &mut TxContext as its last argument, with an optional One Time Witness (OTW) as its first argument.
// 4. It must not return any values.
//
// fun init(ctx: &TxContext) { /* ... */}
// fun init(otw: OTW, ctx: &mut TxContext) { /* ... */ }
//

/// Checks if the given function (by index) is a valid `init` function.
// TODO: Note that we currently trigger a panic if a function named 'init' fails to satisfy certain criteria to qualify as a constructor.
// This behavior is not enforced by the move compiler itself.
pub fn is_init(
    function_id: FunctionId,
    move_function_arguments: &Signature,
    move_function_return: &Signature,
    function_def: &FunctionDefinition,
    datatype_handles_map: &HashMap<DatatypeHandleIndex, UserDefinedType>,
    module: &CompiledModule,
) -> bool {
    // Must be named `init`
    if function_id.identifier != INIT_FUNCTION_NAME {
        return false;
    }

    // Must be private
    assert_eq!(
        function_def.visibility,
        Visibility::Private,
        "init() functions must be private"
    );

    // Must have 1 or 2 arguments
    let arg_count = move_function_arguments.len();
    assert!(
        (1..=2).contains(&arg_count),
        "init() functions must have 1 or 2 arguments"
    );

    // Check TxContext in the last argument
    let is_tx_context_ref = move_function_arguments
        .0
        .last()
        .map(|last| {
            matches!(
                IntermediateType::try_from_signature_token(last, datatype_handles_map).unwrap(),
                IntermediateType::IRef(inner) | IntermediateType::IMutRef(inner)
                    if matches!(
                        inner.as_ref(),
                        IntermediateType::IExternalUserData { module_id, identifier }
                            if TxContext::struct_is_tx_context(module_id, identifier)
                    )
            )
        })
        .unwrap_or(false);

    assert!(
        is_tx_context_ref,
        "Last argument must be &TxContext or &mut TxContext"
    );

    // Check OTW if 2 arguments
    if arg_count == 2 {
        assert!(
            is_one_time_witness(module, &move_function_arguments.0[0]),
            "First argument must be a valid one-time witness type"
        );
    }

    // Must not return any values
    assert!(
        move_function_return.is_empty(),
        "init() functions must return no values"
    );

    true
}

/// Checks if the given signature token is a one-time witness type.
// OTW (One-time witness) types are structs with the following requirements:
// i. Their name is the upper-case version of the module's name.
// ii. They have no fields (or a single boolean field).
// iii. They have no type parameters.
// iv. They have only the 'drop' ability.
pub fn is_one_time_witness(module: &CompiledModule, tok: &SignatureToken) -> bool {
    // 1. Argument must be a struct
    let struct_idx = match tok {
        SignatureToken::Datatype(idx) => idx,
        _ => return false,
    };

    let handle = module.datatype_handle_at(*struct_idx);

    // 2. Name must match uppercase module name
    let module_handle = module.module_handle_at(handle.module);
    let module_name = module.identifier_at(module_handle.name).as_str();
    let struct_name = module.identifier_at(handle.name).as_str();
    if struct_name != module_name.to_ascii_uppercase() {
        return false;
    }

    // 3. Must have only the Drop ability
    if handle.abilities != (AbilitySet::EMPTY | Ability::Drop) {
        return false;
    }

    // 4. Must have no type parameters
    if !handle.type_parameters.is_empty() {
        return false;
    }

    // 5. Must have 0 or 1 field (and if 1, it must be Bool)
    let struct_def = match module
        .struct_defs
        .iter()
        .find(|def| def.struct_handle == *struct_idx)
    {
        Some(def) => def,
        None => return false,
    };

    let field_count = struct_def.declared_field_count().unwrap_or(0);
    if field_count > 1 {
        return false;
    }

    if field_count == 1 {
        let field = struct_def.field(0).unwrap();
        if field.signature.0 != SignatureToken::Bool {
            return false;
        }
    }

    true
}
