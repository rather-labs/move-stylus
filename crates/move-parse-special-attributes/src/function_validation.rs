// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

use move_compiler::{
    diagnostics::codes::{DiagnosticInfo, Severity, custom},
    parser::ast::{Bind_, Exp, Exp_, Function, FunctionBody_, NameAccessChain_, SequenceItem_},
};
use move_ir_types::location::Loc;
use move_symbol_pool::Symbol;

use crate::{
    AbiError, Event, Struct_,
    error::{DIAGNOSTIC_CATEGORY, SpecialAttributeError, SpecialAttributeErrorKind},
    function_modifiers::Signature,
    types::Type,
};

use crate::ModuleId;
use std::collections::{HashMap, HashSet};

/// Maps variable names to the struct type they were bound to.
/// e.g., `let e = ErrorOk(a);` creates a binding `e -> ErrorOk`
pub type VariableBindings = HashMap<Symbol, Symbol>;

/// Maps function names (or aliases) to their original name and source module.
/// e.g., if `use stylus::error::revert as revert_alias;` is in scope,
/// this would contain: `revert_alias` -> ("revert", ModuleId { address: SF_ADDRESS, module_name: "error" })
pub type FunctionAliasMap = HashMap<Symbol, (Symbol, ModuleId)>;

/// Builds a function alias map from imported members.
/// This allows us to resolve function aliases back to their original names and modules.
fn build_function_alias_map(
    imported_members: &HashMap<ModuleId, Vec<(Symbol, Option<Symbol>)>>,
) -> FunctionAliasMap {
    let mut map = FunctionAliasMap::new();

    for (module_id, members) in imported_members {
        for (original_name, alias_opt) in members {
            // Use alias if present, otherwise use the original name
            let key = alias_opt.unwrap_or(*original_name);
            map.insert(key, (*original_name, module_id.clone()));
        }
    }

    map
}

/// Represents an extracted function call from the AST
#[derive(Debug, Clone)]
pub struct ExtractedFunctionCall {
    /// The name of the function being called (e.g., "emit", "revert", "borrow_uid")
    /// For qualified paths like `module::func`, this contains just the function name
    pub function_name: Symbol,
    /// Optional module path (e.g., for `event::emit`, this would be Some("event"))
    /// Reserved for future use to validate calls come from the correct module
    #[allow(dead_code)]
    pub module_path: Option<Vec<Symbol>>,
    /// The arguments passed to the function call
    pub arguments: Vec<Exp>,
    /// Location in source code for error reporting
    pub loc: Loc,
    /// Whether this is a method call (DotCall) vs a direct call
    /// Reserved for future use (e.g., borrow_uid is typically a method call)
    #[allow(dead_code)]
    pub is_method_call: bool,
}

/// Extracts all function calls and variable bindings from a function body by recursively traversing the AST.
///
/// Returns:
/// - A list of extracted function calls
/// - A map of variable bindings (variable name -> struct type it was bound to)
pub fn extract_function_calls(
    function: &Function,
) -> (Vec<ExtractedFunctionCall>, VariableBindings) {
    let mut calls = Vec::new();
    let mut bindings = VariableBindings::new();

    if let FunctionBody_::Defined(sequence) = &function.body.value {
        // pub type Sequence = (
        //     Vec<UseDecl>,        // use declarations at the start of the block
        //     Vec<SequenceItem>,   // the statements in the block
        //     Option<Loc>,         // location of trailing semicolon (if any)
        //     Box<Option<Exp>>,    // the final expression (return value of the block)
        // );
        let (_, sequence_items, _, final_exp) = sequence;

        // Process each sequence item
        for item in sequence_items {
            extract_from_sequence_item(&item.value, &mut calls, &mut bindings);
        }

        // Process the final expression if present
        if let Some(exp) = final_exp.as_ref() {
            extract_from_exp(exp, &mut calls, &mut bindings);
        }
    }

    (calls, bindings)
}

/// Extracts function calls from a sequence item and tracks variable bindings
fn extract_from_sequence_item(
    item: &SequenceItem_,
    calls: &mut Vec<ExtractedFunctionCall>,
    bindings: &mut VariableBindings,
) {
    match item {
        SequenceItem_::Seq(exp) => {
            // e;  -- an expression used as a statement
            extract_from_exp(exp, calls, bindings);
        }
        SequenceItem_::Declare(_, _) => {
            // let b: t;  -- declaration without initialization
            // nothing to extract
        }
        SequenceItem_::Bind(bind_list, _, exp) => {
            // let b: t = e;  -- declaration with initialization
            // Track what struct type the variable is bound to
            if let Some(struct_name) = extract_struct_name_from_exp(exp, bindings) {
                for bind in &bind_list.value {
                    if let Bind_::Var(_, var) = &bind.value {
                        bindings.insert(var.0.value, struct_name);
                    }
                }
            }
            extract_from_exp(exp, calls, bindings);
        }
    }
}

/// Recursively extracts function calls from an expression.
///
/// This focuses on places where `emit()`, `revert()`, and similar calls would realistically appear.
/// We skip expression types where unit-returning functions wouldn't make sense (arithmetic, borrows, etc.)
fn extract_from_exp(
    exp: &Exp,
    calls: &mut Vec<ExtractedFunctionCall>,
    bindings: &mut VariableBindings,
) {
    match &exp.value {
        // === PRIMARY TARGETS: Function calls ===

        // Direct function call: func(args) or module::func(args)
        Exp_::Call(name_chain, args) => {
            let (function_name, module_path) = extract_name_from_chain(&name_chain.value);
            calls.push(ExtractedFunctionCall {
                function_name,
                module_path,
                arguments: args.value.clone(),
                loc: exp.loc,
                is_method_call: false,
            });
            // Also extract from arguments (nested calls)
            for arg in &args.value {
                extract_from_exp(arg, calls, bindings);
            }
        }

        // Method call: obj.method(args)
        Exp_::DotCall(receiver, _dot_loc, method_name, _is_macro, _tyargs, args) => {
            calls.push(ExtractedFunctionCall {
                function_name: method_name.value,
                module_path: None,
                arguments: args.value.clone(),
                loc: exp.loc,
                is_method_call: true,
            });
            extract_from_exp(receiver, calls, bindings);
            for arg in &args.value {
                extract_from_exp(arg, calls, bindings);
            }
        }

        // === CONTROL FLOW: Must recurse into these ===

        // Block: { ... }
        Exp_::Block(sequence) => {
            let (_, sequence_items, _, final_exp) = sequence;
            for item in sequence_items {
                extract_from_sequence_item(&item.value, calls, bindings);
            }
            if let Some(e) = final_exp.as_ref() {
                extract_from_exp(e, calls, bindings);
            }
        }

        // If-else: if (cond) { ... } else { ... }
        Exp_::IfElse(cond, then_branch, else_branch) => {
            extract_from_exp(cond, calls, bindings);
            extract_from_exp(then_branch, calls, bindings);
            if let Some(else_exp) = else_branch {
                extract_from_exp(else_exp, calls, bindings);
            }
        }

        // While loop: while (cond) { ... }
        Exp_::While(cond, body) => {
            extract_from_exp(cond, calls, bindings);
            extract_from_exp(body, calls, bindings);
        }

        // Loop: loop { ... }
        Exp_::Loop(body) => {
            extract_from_exp(body, calls, bindings);
        }

        // Match expression
        Exp_::Match(subject, arms) => {
            extract_from_exp(subject, calls, bindings);
            for arm in &arms.value {
                if let Some(guard) = &arm.value.guard {
                    extract_from_exp(guard, calls, bindings);
                }
                extract_from_exp(&arm.value.rhs, calls, bindings);
            }
        }

        // === WRAPPERS: Just unwrap and recurse ===

        // Labeled expression: 'label: expr
        Exp_::Labeled(_, inner) => extract_from_exp(inner, calls, bindings),

        // Parenthesized expression: (expr)
        Exp_::Parens(inner) => extract_from_exp(inner, calls, bindings),

        // Lambda body: |args| expr
        Exp_::Lambda(_, _, body) => extract_from_exp(body, calls, bindings),

        // Expression list: (e1, e2, ...)
        Exp_::ExpList(exps) => {
            for e in exps {
                extract_from_exp(e, calls, bindings);
            }
        }

        // === SKIP EVERYTHING ELSE ===
        // These are places where emit/revert (which return unit) wouldn't realistically appear:
        // - Arithmetic/logic operands (BinopExp, UnaryExp)
        // - Borrows, dereferences, field access
        // - Casts, type annotations
        // - Vector elements, struct fields
        // - Abort/return/break values
        // - Move/copy expressions
        // - Terminal expressions (values, names, unit)
        _ => {}
    }
}

/// Extracts the function name and optional module path from a NameAccessChain
fn extract_name_from_chain(chain: &NameAccessChain_) -> (Symbol, Option<Vec<Symbol>>) {
    match chain {
        NameAccessChain_::Single(path_entry) => (path_entry.name.value, None),
        NameAccessChain_::Path(name_path) => {
            // For paths like `module::func` or `addr::module::func`
            let mut path_parts: Vec<Symbol> = Vec::new();

            // Extract root name if it's a simple name
            if let move_compiler::parser::ast::LeadingNameAccess_::Name(name) =
                &name_path.root.name.value
            {
                path_parts.push(name.value);
            }

            // Add intermediate entries (all but last are module path)
            if !name_path.entries.is_empty() {
                for entry in &name_path.entries[..name_path.entries.len() - 1] {
                    path_parts.push(entry.name.value);
                }

                // Last entry is the function name
                let func_name = name_path.entries.last().unwrap().name.value;
                let module_path = if path_parts.is_empty() {
                    None
                } else {
                    Some(path_parts)
                };
                (func_name, module_path)
            } else {
                // No entries means root is the function name
                if let move_compiler::parser::ast::LeadingNameAccess_::Name(name) =
                    &name_path.root.name.value
                {
                    (name.value, None)
                } else {
                    // Anonymous or global address - shouldn't happen for function calls
                    (Symbol::from(""), None)
                }
            }
        }
    }
}

/// Extracts the struct name from an expression if it's a struct pack or variable name.
///
/// If the expression is a variable name and we have bindings, we resolve it to the
/// struct type it was bound to. For example, if `let e = ErrorOk(a);`, then
/// `extract_struct_name_from_exp(e, bindings)` returns `ErrorOk`.
fn extract_struct_name_from_exp(exp: &Exp, bindings: &VariableBindings) -> Option<Symbol> {
    match &exp.value {
        // Direct struct instantiation with named fields: MyStruct { field: value, ... }
        Exp_::Pack(name_chain, _) => {
            let (name, _) = extract_name_from_chain(&name_chain.value);
            Some(name)
        }
        // Positional struct instantiation: MyStruct(value1, value2)
        // In Move AST, positional struct constructors look like function calls
        // e.g., `ErrorOk(a)` for `struct ErrorOk(String)` appears as Exp_::Call
        Exp_::Call(name_chain, _) => {
            let (name, _) = extract_name_from_chain(&name_chain.value);
            Some(name)
        }
        // Variable name - resolve to struct type if we have a binding
        Exp_::Name(name_chain) => {
            let (name, _) = extract_name_from_chain(&name_chain.value);
            // If this variable was bound to a struct, return the struct name
            if let Some(struct_name) = bindings.get(&name) {
                Some(*struct_name)
            } else {
                // Otherwise return the name itself (might be a direct struct reference)
                Some(name)
            }
        }
        // Parenthesized expression
        Exp_::Parens(inner) => extract_struct_name_from_exp(inner, bindings),
        // Copy expression
        Exp_::Copy(_, inner) => extract_struct_name_from_exp(inner, bindings),
        // Move expression
        Exp_::Move(_, inner) => extract_struct_name_from_exp(inner, bindings),
        _ => None,
    }
}

/// Checks if a function call is to the native `emit` function from `stylus::event` module.
///
/// This handles both:
/// - Direct imports: `use stylus::event::emit;` then `emit(...)`
/// - Aliased imports: `use stylus::event::emit as emit_alias;` then `emit_alias(...)`
fn is_emit_call(call: &ExtractedFunctionCall, function_alias_map: &FunctionAliasMap) -> bool {
    // Check if this function name (or alias) resolves to emit from stylus::event
    if let Some((original_name, module_id)) = function_alias_map.get(&call.function_name) {
        return original_name.as_str() == "emit"
            && module_id.address == crate::reserved_modules::SF_ADDRESS
            && module_id.module_name.as_str() == "event";
    }

    false
}

/// Checks if a function call is to the native `revert` function from `stylus::error` module.
///
/// This handles both:
/// - Direct imports: `use stylus::error::revert;` then `revert(...)`
/// - Aliased imports: `use stylus::error::revert as revert_alias;` then `revert_alias(...)`
fn is_revert_call(call: &ExtractedFunctionCall, function_alias_map: &FunctionAliasMap) -> bool {
    // Check if this function name (or alias) resolves to revert from stylus::error
    if let Some((original_name, module_id)) = function_alias_map.get(&call.function_name) {
        return original_name.as_str() == "revert"
            && module_id.address == crate::reserved_modules::SF_ADDRESS
            && module_id.module_name.as_str() == "error";
    }

    false
}

/// Validates function calls to emit, revert, and borrow_uid
///
/// - `emit()` must be called with a struct marked as #[event]
/// - `revert()` must be called with a struct marked as #[abi_error]
fn validate_function_calls(
    calls: &[ExtractedFunctionCall],
    events: &HashMap<Symbol, Event>,
    abi_errors: &HashMap<Symbol, AbiError>,
    bindings: &VariableBindings,
    function_alias_map: &FunctionAliasMap,
    _function_loc: Loc,
) -> Result<(), SpecialAttributeError> {
    for call in calls {
        if is_emit_call(call, function_alias_map) {
            // emit() should have exactly one argument that is an event struct
            if call.arguments.is_empty() {
                return Err(SpecialAttributeError {
                    kind: SpecialAttributeErrorKind::FunctionValidation(
                        FunctionValidationError::NativeEmitNoArgument,
                    ),
                    line_of_code: call.loc,
                });
            }

            // Try to extract the struct name from the first argument
            if let Some(struct_name) = extract_struct_name_from_exp(&call.arguments[0], bindings) {
                // Check if the struct is marked as an event
                if !events.contains_key(&struct_name) {
                    return Err(SpecialAttributeError {
                        kind: SpecialAttributeErrorKind::FunctionValidation(
                            FunctionValidationError::NativeEmitNotEventArgument,
                        ),
                        line_of_code: call.loc,
                    });
                }
            } else {
                panic!("struct_name not found for emit call");
            }
        } else if is_revert_call(call, function_alias_map) {
            // revert() should have exactly one argument that is an error struct
            if call.arguments.is_empty() {
                return Err(SpecialAttributeError {
                    kind: SpecialAttributeErrorKind::FunctionValidation(
                        FunctionValidationError::NativeRevertNoArgument,
                    ),
                    line_of_code: call.loc,
                });
            }

            // Try to extract the struct name from the first argument
            if let Some(struct_name) = extract_struct_name_from_exp(&call.arguments[0], bindings) {
                // Check if the struct is marked as an abi_error
                if !abi_errors.contains_key(&struct_name) {
                    return Err(SpecialAttributeError {
                        kind: SpecialAttributeErrorKind::FunctionValidation(
                            FunctionValidationError::NativeRevertNotErrorArgument,
                        ),
                        line_of_code: call.loc,
                    });
                }
            } else {
                panic!("struct_name not found for revert call");
            }
        }
        // Other function calls don't need special validation
    }

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum FunctionValidationError {
    #[error("Only native emit function can take an event struct as an argument")]
    InvalidEventArgument,

    #[error("Only native revert function can take an error struct as an argument")]
    InvalidErrorArgument,

    #[error("Generic functions cannot be entrypoints")]
    GenericFunctionsIsEntry,

    #[error("Entry functions cannot return structs with the key ability")]
    EntryFunctionReturnsKeyStruct,

    #[error("Invalid UID argument. UID is a reserved type and cannot be used as an argument.")]
    InvalidUidArgument,

    #[error(
        "Invalid NamedId argument. NamedId is a reserved type and cannot be used as an argument."
    )]
    InvalidNamedIdArgument,

    #[error("Storage object '{0}' must be a struct with the key ability")]
    StorageObjectNotKeyedStruct(Symbol),

    #[error("Storage object struct '{0}' not found")]
    StorageObjectStructNotFound(Symbol),

    #[error("Parameter '{0}' not found in function signature")]
    ParameterNotFound(Symbol),

    #[error("Struct not found in local or imported modules")]
    StructNotFound,

    #[error("init function cannot be entry")]
    InitFunctionCannotBeEntry,

    #[error("emit() requires an argument that is a struct marked with #[event]")]
    NativeEmitNoArgument,

    #[error("revert() requires an argument that is a struct marked with #[abi_error]")]
    NativeRevertNoArgument,

    #[error("emit() was called with a non-event argument")]
    NativeEmitNotEventArgument,

    #[error("revert() was called with a non-error argument")]
    NativeRevertNotErrorArgument,
}

impl From<&FunctionValidationError> for DiagnosticInfo {
    fn from(value: &FunctionValidationError) -> Self {
        custom(
            DIAGNOSTIC_CATEGORY,
            Severity::BlockingError,
            3,
            3,
            Box::leak(value.to_string().into_boxed_str()),
        )
    }
}

/// Checks if a type is an Event by comparing its name with known events
fn is_event_type(type_: &Type, events: &HashMap<Symbol, Event>) -> bool {
    match type_ {
        Type::UserDataType(name, _) => events.contains_key(name),
        _ => false,
    }
}

/// Checks if a type is an AbiError by comparing its name with known abi_errors
fn is_abi_error_type(type_: &Type, abi_errors: &HashMap<Symbol, AbiError>) -> bool {
    match type_ {
        Type::UserDataType(name, _) => abi_errors.contains_key(name),
        _ => false,
    }
}

/// Checks if a function is the native `emit` function from `stylus::event` module.
///
/// The native emit function has the signature: `public native fun emit<T: copy + drop>(event: T)`
/// and is defined in the stylus framework package.
fn is_native_emit(function: &Function, package_address: [u8; 32]) -> bool {
    function.name.to_string() == "emit"
        && function.body.value == FunctionBody_::Native
        && package_address == crate::reserved_modules::SF_ADDRESS
}

/// Checks if a function is the native `revert` function from `stylus::error` module.
///
/// The native revert function has the signature: `public native fun revert<T: copy + drop>(error: T)`
/// and is defined in the stylus framework package.
fn is_native_revert(function: &Function, package_address: [u8; 32]) -> bool {
    function.name.to_string() == "revert"
        && function.body.value == FunctionBody_::Native
        && package_address == crate::reserved_modules::SF_ADDRESS
}

/// Extracts all struct names from a type (recursively handles vectors, tuples, etc.)
fn extract_struct_names(type_: &Type) -> Vec<Symbol> {
    match type_ {
        Type::UserDataType(name, _) => vec![*name],
        Type::Vector(inner) => extract_struct_names(inner),
        Type::Tuple(types) => types.iter().flat_map(extract_struct_names).collect(),
        _ => Vec::new(),
    }
}

/// Validates that a function is correct:
///
/// - If the function is generic, it cannot be an entrypoint.
/// - If the function has an Event parameter, it must be an emit function; otherwise, it is invalid.
/// - If the function has an AbiError parameter, it must be a revert function; otherwise, it is invalid.
/// - Entry functions cannot return structs with the key ability.
/// - Functions cannot take a UID as arguments, unless it is a function from the Stylus Framework package.
/// - Calls to `emit` must pass an event struct as argument.
/// - Calls to `revert` must pass an error struct as argument.
pub fn validate_function(
    function: &Function,
    events: &HashMap<Symbol, Event>,
    abi_errors: &HashMap<Symbol, AbiError>,
    structs: &[Struct_],
    deps_structs: &HashMap<ModuleId, Vec<Struct_>>,
    imported_members: &HashMap<ModuleId, Vec<(Symbol, Option<Symbol>)>>,
    package_address: [u8; 32],
) -> Result<(), SpecialAttributeError> {
    let signature = crate::function_modifiers::Function::parse_signature(&function.signature);

    // If any of the function's parameters is a UID type and the package address does not match the Stylus Framework address, this function should be rejected as invalid.
    if package_address != crate::reserved_modules::SF_ADDRESS {
        for param in &signature.parameters {
            for struct_name in extract_struct_names(&param.type_) {
                if struct_name.as_str() == "UID" {
                    return Err(SpecialAttributeError {
                        kind: SpecialAttributeErrorKind::FunctionValidation(
                            FunctionValidationError::InvalidUidArgument,
                        ),
                        line_of_code: function.loc,
                    });
                } else if struct_name.as_str() == "NamedId" {
                    return Err(SpecialAttributeError {
                        kind: SpecialAttributeErrorKind::FunctionValidation(
                            FunctionValidationError::InvalidNamedIdArgument,
                        ),
                        line_of_code: function.loc,
                    });
                }
            }
        }
    }

    if function.entry.is_some() {
        // If the function is generic and is entry, it should be rejected as invalid.
        if !function.signature.type_parameters.is_empty() {
            return Err(SpecialAttributeError {
                kind: SpecialAttributeErrorKind::FunctionValidation(
                    FunctionValidationError::GenericFunctionsIsEntry,
                ),
                line_of_code: function.loc,
            });
        }

        // Check if return type contains any structs with the key ability
        for struct_name in extract_struct_names(&signature.return_type) {
            // First, check if the struct exists in local structs
            let module_struct = structs.iter().find(|s| s.name == struct_name);

            // If not defined in the module, check in imported members
            let imported_struct = module_struct
                .is_none()
                .then(|| {
                    imported_members.iter().find_map(|(module_id, members)| {
                        members.iter().find_map(|(original_name, alias_opt)| {
                            // First check the original name, if not found, check the alias
                            if original_name == &struct_name
                                || alias_opt
                                    .as_ref()
                                    .map(|a| a == &struct_name)
                                    .unwrap_or(false)
                            {
                                // If there's a match, search the struct in the dependency's structs hashmap.
                                // This map supplements the imported members by providing extra information about structs, including whether they have the key ability.
                                deps_structs.get(module_id).and_then(|module_structs| {
                                    module_structs.iter().find(|s| s.name == *original_name)
                                })
                            } else {
                                None
                            }
                        })
                    })
                })
                .flatten();

            // If struct is not found in either local or imported, return error
            match module_struct.or(imported_struct) {
                None => {
                    // Note: here we might encounter the case where the datatype is actually an enum not an struct,
                    // in this case we dont want to return an error, we want to ignore it.
                }
                Some(found_struct) => {
                    // If struct is found and has key ability, return error
                    if found_struct.has_key {
                        return Err(SpecialAttributeError {
                            kind: SpecialAttributeErrorKind::FunctionValidation(
                                FunctionValidationError::EntryFunctionReturnsKeyStruct,
                            ),
                            line_of_code: function.loc,
                        });
                    }
                }
            }
        }
    }

    // Event and error types can only be passed as arguments to the native emit/revert functions
    // from the stylus framework. If a non-framework function has an event or error argument, reject it.
    //
    // Note: We skip validation for the native emit/revert functions themselves because they use
    // generic type parameters (e.g., `emit<T: copy + drop>(event: T)`). The generic `T` won't match
    // our `is_event_type` check since it's not a concrete event type registered in the `events` map.

    // Event types can only be passed as arguments to the native `emit` function from `stylus::event` module.
    if !is_native_emit(function, package_address)
        && signature
            .parameters
            .iter()
            .any(|p| is_event_type(&p.type_, events))
    {
        return Err(SpecialAttributeError {
            kind: SpecialAttributeErrorKind::FunctionValidation(
                FunctionValidationError::InvalidEventArgument,
            ),
            line_of_code: function.loc,
        });
    }

    // Error types can only be passed as arguments to the native `revert` function from `stylus::error` module.
    if !is_native_revert(function, package_address)
        && signature
            .parameters
            .iter()
            .any(|p| is_abi_error_type(&p.type_, abi_errors))
    {
        return Err(SpecialAttributeError {
            kind: SpecialAttributeErrorKind::FunctionValidation(
                FunctionValidationError::InvalidErrorArgument,
            ),
            line_of_code: function.loc,
        });
    }

    // Build a set of known struct names to filter out struct constructors from function calls
    // At the parser level, positional struct constructors (e.g., `MyStruct(a)`) look like function calls
    let known_struct_names: HashSet<Symbol> = structs
        .iter()
        .map(|s| s.name)
        .chain(events.keys().copied())
        .chain(abi_errors.keys().copied())
        .collect();

    // Build function alias map for resolving function aliases (e.g., `revert as revert_alias`)
    let function_alias_map = build_function_alias_map(imported_members);

    // Extract calls (this includes function calls and struct constructors), then filter out struct constructors
    let (all_calls, bindings) = extract_function_calls(function);
    let function_calls: Vec<_> = all_calls
        .into_iter()
        .filter(|call| !known_struct_names.contains(&call.function_name))
        .collect();

    validate_function_calls(
        &function_calls,
        events,
        abi_errors,
        &bindings,
        &function_alias_map,
        function.loc,
    )?;

    Ok(())
}

pub fn check_storage_object_param(
    signature: &Signature,
    identifier: Symbol,
    identifier_loc: Loc,
    module_structs: &[Struct_],
) -> Result<(), SpecialAttributeError> {
    if let Some(param_type_name) = signature.parameters.iter().find_map(|p| {
        if p.name == identifier {
            match &p.type_ {
                Type::UserDataType(name, _) => Some(name),
                Type::Ref(inner) => {
                    if let Type::UserDataType(name, _) = &**inner {
                        Some(name)
                    } else {
                        None
                    }
                }
                Type::MutRef(inner) => {
                    if let Type::UserDataType(name, _) = &**inner {
                        Some(name)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }) {
        if let Some(struct_) = module_structs.iter().find(|s| s.name == *param_type_name) {
            if !struct_.has_key {
                return Err(SpecialAttributeError {
                    kind: SpecialAttributeErrorKind::FunctionValidation(
                        FunctionValidationError::StorageObjectNotKeyedStruct(identifier),
                    ),
                    line_of_code: identifier_loc,
                });
            }
        } else {
            return Err(SpecialAttributeError {
                kind: SpecialAttributeErrorKind::FunctionValidation(
                    FunctionValidationError::StorageObjectStructNotFound(identifier),
                ),
                line_of_code: identifier_loc,
            });
        }
    } else {
        return Err(SpecialAttributeError {
            kind: SpecialAttributeErrorKind::FunctionValidation(
                FunctionValidationError::ParameterNotFound(identifier),
            ),
            line_of_code: identifier_loc,
        });
    }

    Ok(())
}

pub fn check_repeated_storage_object_param(
    processed_storage_objects: &mut HashSet<Symbol>,
    identifier: Symbol,
    identifier_loc: Loc,
) -> Result<(), SpecialAttributeError> {
    if processed_storage_objects.contains(&identifier) {
        Err(SpecialAttributeError {
            kind: SpecialAttributeErrorKind::RepeatedStorageObject(identifier),
            line_of_code: identifier_loc,
        })
    } else {
        // Add to processed storage objects
        processed_storage_objects.insert(identifier);
        Ok(())
    }
}
