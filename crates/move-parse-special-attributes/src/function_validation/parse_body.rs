// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

//! AST extraction utilities for function call analysis.
//!
//! This module provides functions to traverse the Move AST and extract:
//! - Function calls (both direct and method calls)
//! - Variable bindings (for tracking struct types bound to variables)

use move_compiler::parser::ast::{
    Bind_, Exp, Exp_, Function, FunctionBody_, NameAccessChain_, SequenceItem_,
};
use move_ir_types::location::Loc;
use move_symbol_pool::Symbol;
use std::collections::HashMap;

/// Maps variable names to the struct type they were bound to.
/// e.g., `let e = ErrorOk(a);` creates a binding `e -> ErrorOk`
pub type VariableBindings = HashMap<Symbol, Symbol>;

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
pub fn extract_name_from_chain(chain: &NameAccessChain_) -> (Symbol, Option<Vec<Symbol>>) {
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
pub fn extract_struct_name_from_exp(exp: &Exp, bindings: &VariableBindings) -> Option<Symbol> {
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
