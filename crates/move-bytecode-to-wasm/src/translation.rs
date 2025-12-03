use bytecodes::dynamic_fields::add_field_borrow_mut_global_var_instructions;
pub use error::TranslationError;

pub(crate) mod bytecodes;
pub(crate) mod flow;
pub(crate) mod types_stack;

pub mod error;
pub mod functions;
/// The types in this module represent an intermediate Rust representation of Move types
/// that is used to generate the WASM code.
pub mod intermediate_types;
pub mod table;

use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use walrus::{
    FunctionBuilder, FunctionId as WasmFunctionId, GlobalId, InstrSeqBuilder, LocalId, Module,
    TableId, ValType,
    ir::{BinaryOp, InstrSeqId, InstrSeqType, LoadKind, MemArg, StoreKind, UnaryOp},
};

use relooper::BranchMode;

use move_binary_format::{
    file_format::{Bytecode, CodeUnit},
    internals::ModuleIndex,
};

use crate::{
    CompilationContext, GlobalFunctionTable,
    abi_types::error_encoding::build_abort_error_message,
    compilation_context::{ModuleData, ModuleId},
    data::DATA_ABORT_MESSAGE_PTR_OFFSET,
    generics::{replace_type_parameters, type_contains_generics},
    hostio::host_functions::storage_flush_cache,
    native_functions::NativeFunction,
    runtime::RuntimeFunction,
    vm_handled_types::{self, VmHandledType, named_id::NamedId, uid::Uid},
    wasm_builder_extensions::WasmBuilderExtension,
};

use flow::Flow;

use functions::{
    MappedFunction, add_unpack_function_return_values_instructions, prepare_function_arguments,
    prepare_function_return,
};

use intermediate_types::{
    IntermediateType, VmHandledStruct,
    error::IntermediateTypeError,
    heap_integers::{IU128, IU256},
    simple_integers::{IU8, IU16, IU32, IU64},
    structs::IStruct,
    vector::IVector,
};

use table::{FunctionId, FunctionTable, TableEntry};

use functions::JumpTableData;
use types_stack::{TypesStack, TypesStackError};

#[derive(Copy, Clone)]
struct SimpleScope {
    simple_block_id: InstrSeqId,
    next_label: Option<u16>, // next == Some(label) if next is Simple
}

#[derive(Default)]
pub struct ControlTargets {
    // Loop control targets
    loop_continue: HashMap<u16, InstrSeqId>,
    loop_break: HashMap<u16, InstrSeqId>,
    // Ancestor stack of enclosing Simples for MergedBranch resolution
    simple_scopes: Vec<SimpleScope>,
}

impl ControlTargets {
    pub fn new() -> Self {
        Self {
            loop_continue: HashMap::new(),
            loop_break: HashMap::new(),
            simple_scopes: Vec::new(),
        }
    }

    /* ---------- Simple scopes (for MergedBranch) ---------- */

    /// Call when you open the wrapping block of a `Flow::Simple`.
    pub fn push_simple_scope(&mut self, simple_block_id: InstrSeqId, next_label: Option<u16>) {
        self.simple_scopes.push(SimpleScope {
            simple_block_id,
            next_label,
        });
    }

    /// Call right before you close that wrapping block.
    pub fn pop_simple_scope(&mut self) {
        self.simple_scopes.pop();
    }

    /// Implement the “walk up to nearest Simple whose next == label” rule.
    fn resolve_merged(&self, target_label: u16) -> Option<InstrSeqId> {
        for scope in self.simple_scopes.iter().rev() {
            if scope.next_label == Some(target_label) {
                return Some(scope.simple_block_id);
            }
        }
        None
    }

    /* ---------- Loop scopes ---------- */

    /// Call when you open `block { loop { ... } }` for a given `loop_id`.
    pub fn set_loop_targets(
        &mut self,
        loop_id: u16,
        break_target: InstrSeqId,
        continue_target: InstrSeqId,
    ) {
        self.loop_break.insert(loop_id, break_target);
        self.loop_continue.insert(loop_id, continue_target);
    }

    /// Optional: call when leaving the loop scope (not strictly necessary if loop_ids are unique).
    pub fn clear_loop_targets(&mut self, loop_id: u16) {
        self.loop_break.remove(&loop_id);
        self.loop_continue.remove(&loop_id);
    }

    /* ---------- Unified branch resolver ---------- */

    pub fn resolve(
        &self,
        mode: BranchMode,
        label: u16,
    ) -> Result<Option<InstrSeqId>, TranslationError> {
        let flow_mode = match mode {
            BranchMode::MergedBranch | BranchMode::MergedBranchIntoMulti => {
                self.resolve_merged(label)
            }
            BranchMode::LoopBreak(loop_id) | BranchMode::LoopBreakIntoMulti(loop_id) => self
                .resolve_merged(label)
                .or_else(|| self.loop_break.get(&loop_id).copied()),
            BranchMode::LoopContinue(loop_id) | BranchMode::LoopContinueIntoMulti(loop_id) => {
                self.loop_continue.get(&loop_id).copied()
            }
            _ => return Err(TranslationError::UnssuportedBranchMode(mode)),
        };

        Ok(flow_mode)
    }
}

#[derive(Debug)]
struct StorageIdParentInformation {
    module_id: ModuleId,
    index: u16,
    instance_types: Option<Vec<IntermediateType>>,
}

/// This is used to pass around the context of the translation process. Also clippy complains about too many arguments in translate_instruction.
struct TranslateFlowContext<'a> {
    compilation_ctx: &'a CompilationContext<'a>,
    module_data: &'a ModuleData,
    types_stack: &'a mut TypesStack,
    function_information: &'a MappedFunction,
    function_table: &'a mut FunctionTable,
    function_locals: &'a Vec<LocalId>,
    uid_locals: &'a mut HashMap<u8, StorageIdParentInformation>,
    control_targets: &'a mut ControlTargets,
    jump_table: &'a mut Option<JumpTableData>,
    dynamic_fields_global_variables: &'a mut Vec<(GlobalId, IntermediateType)>,
}

/// Translates a move function to WASM
///
/// The return values are:
/// 1. The translated WASM FunctionId
/// 2. A list of function ids from other modules to be translated and linked.
pub fn translate_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    module_data: &ModuleData,
    function_table: &mut FunctionTable,
    function_information: &MappedFunction,
    move_bytecode: &CodeUnit,
    dynamic_fields_global_variables: &mut Vec<(GlobalId, IntermediateType)>,
) -> Result<(WasmFunctionId, HashSet<FunctionId>), TranslationError> {
    let params = function_information.signature.get_argument_wasm_types()?;
    let results = function_information.signature.get_return_wasm_types();
    let mut function = FunctionBuilder::new(&mut module.types, &params, &results);

    #[cfg(debug_assertions)]
    function.name(function_information.function_id.identifier.clone());

    println!("{}", function_information.function_id.identifier);

    let mut builder = function.func_body();

    let (arguments, locals) = process_fn_local_variables(function_information, module)?;

    // All the function locals are compose by the argument locals concatenated with the local
    // variable locals
    let mut function_locals = Vec::new();
    function_locals.extend_from_slice(&arguments);
    function_locals.extend_from_slice(&locals);
    box_args(
        &mut builder,
        module,
        compilation_ctx,
        &mut function_locals,
        function_information,
    )?;

    let flow = Flow::new(move_bytecode)?;

    let mut types_stack = TypesStack::new();
    let mut functions_to_link = HashSet::new();
    let mut uid_locals: HashMap<u8, StorageIdParentInformation> = HashMap::new();
    let mut control_targets = ControlTargets::new();

    let mut ctx = TranslateFlowContext {
        compilation_ctx,
        module_data,
        function_table,
        function_information,
        function_locals: &function_locals,
        uid_locals: &mut uid_locals,
        types_stack: &mut types_stack,
        control_targets: &mut control_targets,
        jump_table: &mut None, // The current jump table we are translating
        dynamic_fields_global_variables,
    };

    translate_flow(
        &mut ctx,
        &mut builder,
        module,
        &flow,
        &mut functions_to_link,
    )?;

    let function_id = function.finish(arguments, &mut module.funcs);

    Ok((function_id, functions_to_link))
}

/// Recusively translate the flow to wasm.
/// It is responsible for both handling the control flow as well as translating the specific instructions to wasm.
fn translate_flow(
    ctx: &mut TranslateFlowContext,
    builder: &mut InstrSeqBuilder,
    module: &mut Module,
    flow: &Flow,
    functions_to_link: &mut HashSet<FunctionId>,
) -> Result<(), TranslationError> {
    match flow {
        Flow::Simple {
            instructions,
            branches,
            immediate,
            next,
            ..
        } => {
            // If the immediate flow contains a Ret instruction, set the result type of the block to the function's return type.
            let ty = InstrSeqType::new(
                &mut module.types,
                &[],
                if instructions
                    .last()
                    .is_some_and(|b| matches!(b, Bytecode::Ret))
                    || immediate.dominates_return()
                {
                    &ctx.function_information.results
                } else {
                    &[]
                },
            );

            let mut inner_result = Ok(());
            builder.block(ty, |block| {
                // Add the simple scope to the control targets.
                ctx.control_targets.push_simple_scope(
                    block.id(),
                    match &**next {
                        Flow::Simple { label, .. } => Some(*label),
                        _ => None,
                    },
                );

                // First translate the instuctions associated with the simple flow itself
                for instruction in *instructions {
                    match translate_instruction(instruction, ctx, block, module, branches) {
                        Ok(mut fns_to_link) => {
                            functions_to_link.extend(fns_to_link.drain(..));
                        }
                        Err(e) => {
                            inner_result = Err(TranslationError::AtInstruction(
                                instruction.clone(),
                                Rc::new(e),
                            ));
                        }
                    }
                }
                if inner_result.is_ok() {
                    // Translate the immediate flow within the current scope
                    inner_result = translate_flow(ctx, block, module, immediate, functions_to_link);
                }

                // Done with this Simple's inner region. Pop the simple scope.
                ctx.control_targets.pop_simple_scope();
            });
            inner_result?;
            // Translate the next flow outside the current scope
            translate_flow(ctx, builder, module, next, functions_to_link)?;
        }
        Flow::Loop {
            loop_id,
            inner,
            next,
            ..
        } => {
            // If the inner flow contains a Ret instruction, set the result type of the block to the function's return type.
            let ty = InstrSeqType::new(
                &mut module.types,
                &[],
                if inner.dominates_return() {
                    &ctx.function_information.results
                } else {
                    &[]
                },
            );

            // We wrap the loop in a block so we have a "landing spot" if we need to break out of it
            // (in case we encounter a BranchMode::LoopBreak).
            let mut inner_result = Ok(());
            builder.block(ty, |block| {
                let block_id = block.id();
                block.loop_(ty, |loop_| {
                    // Add the loop targets to the control targets.
                    ctx.control_targets
                        .set_loop_targets(*loop_id, block_id, loop_.id());

                    // Translate the loop body (inner) inside the loop block.
                    inner_result = translate_flow(ctx, loop_, module, inner, functions_to_link);

                    // Clear the loop targets after the loop body is translated.
                    ctx.control_targets.clear_loop_targets(*loop_id);
                });
            });

            inner_result?;

            // Translate the next flow outside the wrapping block.
            translate_flow(ctx, builder, module, next, functions_to_link)?;
        }
        Flow::IfElse {
            then_body,
            else_body,
            ..
        } => {
            // When the if/else flow stems from a two-branch match on an enum, the jump_table will be Some.
            if let Some(jump_table) = ctx.jump_table.take() {
                let enum_data = ctx
                    .module_data
                    .enums
                    .get_enum_by_index(jump_table.enum_index as u16)?;

                // The jump_table should have exactly 2 entries in this case.
                if jump_table.offsets.len() != 2 {
                    return Err(TranslationError::IfElseJumpTableBranchesNumberMismatch);
                }

                // If offset[0] < offset[1], it means the `consequent` block corresponds to the first variant
                // and the `alternative` block corresponds to the second variant.
                // In that case the first variant index should match the value on the stack.
                if jump_table.offsets[0] <= jump_table.offsets[1] {
                    builder.i32_const(enum_data.variants[0].index as i32);
                } else {
                    builder.i32_const(enum_data.variants[1].index as i32);
                };
                builder.binop(BinaryOp::I32Eq);
            }

            let condition = module.locals.add(ValType::I32);
            builder.local_set(condition);

            let then_ty = if then_body.dominates_return() {
                ctx.function_information.results.clone()
            } else {
                vec![]
            };

            let else_ty = if else_body.dominates_return() {
                ctx.function_information.results.clone()
            } else {
                vec![]
            };

            if then_ty == else_ty {
                // CASE 1: both arms have the same result type (often empty)
                let join_ty = InstrSeqType::new(&mut module.types, &[], &then_ty);

                let mut inner_result = Ok(());
                builder.block(join_ty, |join| {
                    let join_id = join.id();
                    join.block(None::<ValType>, |guard| {
                        guard.local_get(condition);
                        guard.br_if(guard.id());
                        // ELSE (inside guard)
                        inner_result =
                            translate_flow(ctx, guard, module, else_body, functions_to_link);
                        guard.br(join_id); // reconverge
                    });
                    if inner_result.is_ok() {
                        // THEN (after guard)
                        inner_result =
                            translate_flow(ctx, join, module, then_body, functions_to_link);
                    }
                });
                inner_result?;
            } else if !then_ty.is_empty() && else_ty.is_empty() {
                // CASE 2: ONLY THEN yields values; ELSE is empty
                let join_ty = InstrSeqType::new(&mut module.types, &[], &then_ty);

                let mut inner_result = Ok(());
                builder.block(join_ty, |join| {
                    join.block(None::<ValType>, |guard| {
                        guard.local_get(condition);
                        guard.br_if(guard.id());
                        // ELSE (no result) inside guard
                        inner_result =
                            translate_flow(ctx, guard, module, else_body, functions_to_link);
                    });
                    // THEN (produces join result) after guard
                    if inner_result.is_ok() {
                        inner_result =
                            translate_flow(ctx, join, module, then_body, functions_to_link);
                    }
                });
                inner_result?;
            } else if then_ty.is_empty() && !else_ty.is_empty() {
                // CASE 3: ONLY ELSE yields values; THEN is empty
                let join_ty = InstrSeqType::new(&mut module.types, &[], &else_ty);

                let mut inner_result = Ok(());
                builder.block(join_ty, |join| {
                    join.block(None::<ValType>, |guard| {
                        guard.local_get(condition);
                        guard.unop(UnaryOp::I32Eqz); // flip so true => ELSE
                        guard.br_if(guard.id());
                        // THEN (no result) inside guard
                        inner_result =
                            translate_flow(ctx, guard, module, then_body, functions_to_link);
                    });
                    // ELSE (produces join result) after guard
                    if inner_result.is_ok() {
                        inner_result =
                            translate_flow(ctx, join, module, else_body, functions_to_link);
                    }
                });
                inner_result?;
            } else {
                // Both arms yield but with different types → no valid Wasm join
                return Err(TranslationError::IfElseMismatch(
                    then_ty.clone(),
                    else_ty.clone(),
                ));
            }
        }
        Flow::Switch { cases } => {
            let mut cases = cases.clone();

            // label -> enclosing block id map for br_table targets
            let mut label_to_block: HashMap<u16, InstrSeqId> = HashMap::new();

            // Helper: open nested case targets, emit br_table once all targets exist,
            // then translate each case body *after* its label target.
            fn open_cases(
                builder: &mut InstrSeqBuilder,
                module: &mut Module,
                ctx: &mut TranslateFlowContext,
                functions_to_link: &mut HashSet<FunctionId>,
                cases: &[&Flow], // reverse nesting order
                label_to_block: &mut HashMap<u16, InstrSeqId>,
                case_index: usize,
            ) -> Result<(), TranslationError> {
                let ty = InstrSeqType::new(&mut module.types, &[ValType::I32], &[]);
                if case_index == cases.len() {
                    // The jump table is a vector of case labels (i.e. code offsets) in the order they were generated by the Move compiler.
                    // We must stick to this order to switch to the correct case.
                    let jump_table = ctx
                        .jump_table
                        .take()
                        .ok_or(TranslationError::JumpTableNotFound)?;

                    // Build targets in exact jump-table order
                    let mut targets = Vec::with_capacity(jump_table.offsets.len());
                    for &label in &jump_table.offsets {
                        let id = *label_to_block
                            .get(&label)
                            .ok_or(TranslationError::MissingBlockIdForJumpTableLabel)?;

                        targets.push(id);
                    }

                    // Out-of-range → fall into a trap block
                    builder.block(ty, |trap| {
                        trap.br_table(targets.into_boxed_slice(), trap.id());
                    });
                    builder.unreachable();
                    return Ok(());
                }

                let mut inner_result: Result<(), TranslationError> = Ok(());
                builder.block(ty, |case_block| {
                    inner_result = (|| {
                        label_to_block.insert(cases[case_index].get_label()?, case_block.id());
                        open_cases(
                            case_block,
                            module,
                            ctx,
                            functions_to_link,
                            cases,
                            label_to_block,
                            case_index + 1,
                        )?;

                        Ok(())
                    })();
                });

                inner_result?;

                // Emit the body *after* its target
                translate_flow(ctx, builder, module, cases[case_index], functions_to_link)?;

                Ok(())
            }

            // Check out if any of the Switch cases returns a value (contains a `Ret` instruction).
            // - If the function returns something, find the single yielding case.
            // - If more than one case returns a value, panic (this should never happen, as Move creates a merge block
            // where those cases converge and where the actual value is pushed to the stack).
            // - If the function doesn't return anything, use Empty for yielding_case.
            let yielding_case = if !ctx.function_information.results.is_empty() {
                let mut found = None;
                for (i, c) in cases.iter().enumerate() {
                    if c.dominates_return() {
                        if found.is_some() {
                            return Err(TranslationError::SwitchMoreThanOneCase);
                        }
                        found = Some(i);
                    }
                }
                match found {
                    Some(i) => Box::new(cases.remove(i)),
                    None => Box::new(Flow::Empty),
                }
            } else {
                Box::new(Flow::Empty)
            };

            let case_ty = InstrSeqType::new(&mut module.types, &[ValType::I32], &[]);

            // Open a block for the yielding case.
            let mut inner_result: Result<(), TranslationError> = Ok(());
            builder.block(case_ty, |yielding_block| {
                inner_result = (|| {
                    // Create targets deepest-first by iterating cases in reverse
                    let cases_rev: Vec<&Flow> = cases.iter().rev().collect();

                    // If the yielding case is not empty, map its label to the yielding block
                    if !matches!(*yielding_case, Flow::Empty) {
                        label_to_block.insert(yielding_case.get_label()?, yielding_block.id());
                    }

                    // Build target labels and emit br_table; each case body is emitted after its label
                    open_cases(
                        yielding_block,
                        module,
                        ctx,
                        functions_to_link,
                        &cases_rev,
                        &mut label_to_block,
                        0,
                    )?;

                    Ok(())
                })();
            });

            inner_result?;

            // Emit yielding case body if any
            translate_flow(ctx, builder, module, &yielding_case, functions_to_link)?;
        }
        Flow::Empty => (),
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn translate_instruction(
    instruction: &Bytecode,
    translate_flow_ctx: &mut TranslateFlowContext,
    builder: &mut InstrSeqBuilder,
    module: &mut Module,
    branches: &HashMap<u16, BranchMode>,
) -> Result<Vec<FunctionId>, TranslationError> {
    let mut functions_calls_to_link = Vec::new();

    let compilation_ctx = &translate_flow_ctx.compilation_ctx;
    let module_data = &translate_flow_ctx.module_data;
    let mapped_function = &translate_flow_ctx.function_information;
    let function_table = &mut translate_flow_ctx.function_table;
    let types_stack = &mut translate_flow_ctx.types_stack;
    let function_locals = &translate_flow_ctx.function_locals;
    let uid_locals = &mut translate_flow_ctx.uid_locals;
    let control_targets = &translate_flow_ctx.control_targets;
    let jump_table = &mut translate_flow_ctx.jump_table;
    let dynamic_fields_global_variables = &mut translate_flow_ctx.dynamic_fields_global_variables;

    match instruction {
        // Load a fixed constant
        Bytecode::LdConst(global_index) => {
            let constant = &module_data.constants[global_index.0 as usize];
            let mut data = constant.data.clone().into_iter();
            let constant_type = &constant.type_;
            let constant_type: IntermediateType = IntermediateType::try_from_signature_token(
                constant_type,
                &module_data.datatype_handles_map,
            )?;

            constant_type.load_constant_instructions(
                module,
                builder,
                &mut data,
                compilation_ctx,
            )?;

            types_stack.push(constant_type);
            if data.next().is_some() {
                return Err(TranslationError::ConstantDataNotConsumed);
            }
        }
        // Load literals
        Bytecode::LdFalse => {
            builder.i32_const(0);
            types_stack.push(IntermediateType::IBool);
        }
        Bytecode::LdTrue => {
            builder.i32_const(1);
            types_stack.push(IntermediateType::IBool);
        }
        Bytecode::LdU8(literal) => {
            builder.i32_const(*literal as i32);
            types_stack.push(IntermediateType::IU8);
        }
        Bytecode::LdU16(literal) => {
            builder.i32_const(*literal as i32);
            types_stack.push(IntermediateType::IU16);
        }
        Bytecode::LdU32(literal) => {
            builder.i32_const(*literal as i32);
            types_stack.push(IntermediateType::IU32);
        }
        Bytecode::LdU64(literal) => {
            builder.i64_const(*literal as i64);
            types_stack.push(IntermediateType::IU64);
        }
        Bytecode::LdU128(literal) => {
            bytecodes::constants::load_literal_heap_type_to_memory(
                module,
                builder,
                compilation_ctx,
                &literal.to_le_bytes(),
            )?;
            types_stack.push(IntermediateType::IU128);
        }
        Bytecode::LdU256(literal) => {
            bytecodes::constants::load_literal_heap_type_to_memory(
                module,
                builder,
                compilation_ctx,
                &literal.to_le_bytes(),
            )?;
            types_stack.push(IntermediateType::IU256);
        }
        // Function calls
        Bytecode::CallGeneric(function_instantiation_handle_index) => {
            let function_id = &module_data.functions.generic_calls
                [function_instantiation_handle_index.into_index()];

            // Obtain the generic function information
            let function_information = {
                let dependency_data =
                    compilation_ctx.get_module_data_by_id(&function_id.module_id)?;

                dependency_data
                    .functions
                    .get_information_by_identifier(&function_id.identifier)?
            };

            if NamedId::is_remove_function(
                &function_information.function_id.module_id,
                &function_information.function_id.identifier,
            ) {
                types_stack::match_types!((
                    IntermediateType::IGenericStructInstance {
                        module_id: _,
                        index: _,
                        types: _,
                        vm_handled_struct: VmHandledStruct::StorageId {
                            parent_module_id,
                            parent_index,
                            instance_types,
                        }
                    },
                    "struct",
                    types_stack.pop()?
                ));

                let parent_struct = if let Some(instance_types) = instance_types {
                    IntermediateType::IGenericStructInstance {
                        module_id: parent_module_id,
                        index: parent_index,
                        types: instance_types,
                        vm_handled_struct: VmHandledStruct::None,
                    }
                } else {
                    IntermediateType::IStruct {
                        module_id: parent_module_id,
                        index: parent_index,
                        vm_handled_struct: VmHandledStruct::None,
                    }
                };

                let delete_fn = RuntimeFunction::DeleteFromStorage.get_generic(
                    module,
                    compilation_ctx,
                    &[&parent_struct],
                )?;

                // At this point, in the stack que have the pointer to the Uid struct, but what we
                // really need is the pointer to the struct that holds that UId. The struct ptr can
                // be found 4 bytes before the Uid ptr
                builder.i32_const(4).binop(BinaryOp::I32Sub).load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

                builder.call(delete_fn);
            } else {
                let type_instantiations = function_id
                    .type_instantiations
                    .as_ref()
                    .ok_or(TranslationError::CallGenericWihtoutTypeInstantiations)?;

                // If the type_instantiations contains generic parameters, those generic parameters
                // refer to instantiations whithin this context. Instantiatons are obtained using
                // the caller's function type instances (located in
                // `mapped_function.function_id.type_instantiations`)
                let function_information = if type_instantiations.iter().any(type_contains_generics)
                {
                    // Here we extract the type instances from the caller's type instantiations.
                    // Consider the following example:
                    //
                    // ```move
                    // public fun two_generics<T, U>(): Option<U> {
                    //     option::none()
                    // }
                    //
                    // public fun test(): Option<u16> {
                    //     two_generics<u32, u16>()
                    // }
                    // ```
                    //
                    // where `option::none()` is defined as:
                    //
                    // ```move
                    // public fun none<V>(): Option<V> {
                    //     Option { vec: vector::empty() }
                    // }
                    // ```
                    //
                    // In this case:
                    //
                    // - The call to `two_generics` is instantiated with two types: `u32` mapped to
                    //   `ITypeParameter(0)` and `u16` mapped to `ITypeParameter(1)`.
                    //
                    // - `two_generics<T, U>` returns `U`, which corresponds to `ITypeParameter(1)`.
                    //
                    // - `option::none<V>()` has a single type parameter `V`, represented as
                    //   `ITypeParameter(0)`.
                    //
                    // The substitutions happen as follows:
                    //
                    // - Since `option::none()` provides the return value, its parameter
                    //   `V: ITypeParameter(0)` is instantiated with the caller's parameter
                    //   `U: ITypeParameter(1)`.
                    //
                    // - In `test`, we call `two_generics` with `T = u32` and `U = u16`. Therefore:
                    //   - `ITypeParameter(0)` is replaced with `u32`
                    //   - `ITypeParameter(1)` is replaced with `u16`
                    //
                    // If we follow the call chain:
                    //
                    // - `ITypeParameter(0)` (from `option::none`) is replaced with
                    //   `ITypeParameter(1)` (from `two_generics`).
                    //
                    // - `ITypeParameter(1)` is then replaced with `u16` (from the instantiation
                    //   in `test`).
                    //
                    // By transitivity, we infer that the type of `option::none()` in this context
                    // is `u16`.
                    if let Some(caller_type_instances) =
                        &mapped_function.function_id.type_instantiations
                    {
                        let mut instantiations = Vec::new();
                        for field in type_instantiations {
                            instantiations
                                .push(replace_type_parameters(field, caller_type_instances));
                        }

                        function_information.instantiate(&instantiations)
                    }
                    // This should never happen
                    else {
                        return Err(TranslationError::CouldNotInstantiateGenericTypes);
                    }
                } else {
                    function_information.instantiate(type_instantiations)
                };

                // Shadow the function_id variable because now it contains concrete types
                let function_id = &function_information.function_id;
                let arguments = &function_information.signature.arguments;

                let (_argument_types, mut_ref_vec_locals) = prepare_function_arguments(
                    module,
                    builder,
                    arguments,
                    compilation_ctx,
                    types_stack,
                )?;

                // If the function IS native, we link it and call it directly
                if function_information.is_native {
                    let type_instantiations = function_information
                        .function_id
                        .type_instantiations
                        .as_ref()
                        .ok_or(TranslationError::CallGenericWihtoutTypeInstantiations)?;

                    let native_function_id = NativeFunction::get_generic(
                        &function_id.identifier,
                        module,
                        compilation_ctx,
                        &function_id.module_id,
                        type_instantiations,
                    )?;

                    builder.call(native_function_id);
                } else {
                    let table_id = function_table.get_table_id();

                    // If the function is in the table we call it directly
                    let f_entry = if let Some(f) = function_table.get_by_function_id(function_id) {
                        f
                    }
                    // Otherwise, we add it to the table and declare it for translating and linking
                    // before calling it
                    else {
                        functions_calls_to_link.push(function_id.clone());
                        function_table.add(module, function_id.clone(), &function_information)?
                    };

                    call_indirect(
                        f_entry,
                        &function_information.signature.returns,
                        table_id,
                        builder,
                        module,
                        compilation_ctx,
                    )?;

                    // Every time `dynamic_fields::borrow_mut` or
                    // `dynamic_fields_named_id::borrow_mut` are called, we must register a unique
                    // global variable that will hold a `Field` pointer, used at the end of the
                    // router to commit the changes made to the field to storage
                    if vm_handled_types::dynamic_fields::Field::is_borrow_mut_fn(
                        &function_id.module_id,
                        &function_id.identifier,
                    ) || vm_handled_types::table::Table::is_borrow_mut_fn(
                        &function_id.module_id,
                        &function_id.identifier,
                    ) {
                        add_field_borrow_mut_global_var_instructions(
                            module,
                            compilation_ctx,
                            builder,
                            dynamic_fields_global_variables,
                            function_id,
                        )?;
                    }

                    // After the call, check if any argument is a mutable reference to a vector
                    // and update it if needed.
                    if !mut_ref_vec_locals.is_empty() {
                        let update_mut_ref_fn =
                            RuntimeFunction::VecUpdateMutRef.get(module, Some(compilation_ctx))?;
                        for &local in mut_ref_vec_locals.iter() {
                            builder.local_get(local).call(update_mut_ref_fn);
                        }
                    }
                };

                // Insert in the stack types the types returned by the function (if any)
                types_stack.append(&function_information.signature.returns);
            }
        }
        // Function calls
        Bytecode::Call(function_handle_index) => {
            println!("===> {} {:?}", module_data.id, function_handle_index);
            let function_id = &module_data.functions.calls[function_handle_index.into_index()];
            let arguments = &module_data.functions.arguments[function_handle_index.into_index()];

            let function_information = if let Some(fi) = module_data
                .functions
                .information
                .get(function_handle_index.into_index())
            {
                fi
            } else {
                let dependency_data =
                    compilation_ctx.get_module_data_by_id(&function_id.module_id)?;

                dependency_data
                    .functions
                    .get_information_by_identifier(&function_id.identifier)?
            };

            // There are some functions that need to be specially handled, if we find one of those
            // functions, we introduce custom code, otherwise proceed with a normal function call
            if Uid::is_delete_function(
                &function_information.function_id.module_id,
                &function_information.function_id.identifier,
            ) {
                types_stack::match_types!((
                    IntermediateType::IStruct {
                        module_id: _,
                        index: _,
                        vm_handled_struct: VmHandledStruct::StorageId {
                            parent_module_id,
                            parent_index,
                            instance_types,
                        }
                    },
                    "struct",
                    types_stack.pop()?
                ));

                let parent_struct = if let Some(instance_types) = instance_types {
                    IntermediateType::IGenericStructInstance {
                        module_id: parent_module_id,
                        index: parent_index,
                        types: instance_types,
                        vm_handled_struct: VmHandledStruct::None,
                    }
                } else {
                    IntermediateType::IStruct {
                        module_id: parent_module_id,
                        index: parent_index,
                        vm_handled_struct: VmHandledStruct::None,
                    }
                };

                let delete_fn = RuntimeFunction::DeleteFromStorage.get_generic(
                    module,
                    compilation_ctx,
                    &[&parent_struct],
                )?;

                // At this point, in the stack que have the pointer to the Uid struct, but what we
                // really need is the pointer to the struct that holds that UId. The struct ptr can
                // be found 4 bytes before the Uid ptr
                builder.i32_const(4).binop(BinaryOp::I32Sub).load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

                builder.call(delete_fn);
            } else {
                let (argument_types, mut_ref_vec_locals) = prepare_function_arguments(
                    module,
                    builder,
                    arguments,
                    compilation_ctx,
                    types_stack,
                )?;

                // If the function IS native, we link it and call it directly
                if function_information.is_native {
                    let native_function_module = compilation_ctx
                        .get_module_data_by_id(&function_information.function_id.module_id)?;

                    // If the function is a call to another contract, we save the state of the
                    // storage objects before calling it
                    let native_function_id =
                        if native_function_module.is_external_call(&function_id.identifier) {
                            // Along with the argument types, we need to collect the NamedId's that
                            // appear in this function, since in the external call's signature they
                            // are not present, an we will not be able to update them if they
                            // change.
                            let (named_ids_types, named_ids_locals) =
                                get_storage_structs_with_named_ids(
                                    mapped_function,
                                    compilation_ctx,
                                    function_locals,
                                )?
                                .into_iter()
                                .fold(
                                    (Vec::new(), Vec::new()),
                                    |(mut itypes, mut locals), (itype, local)| {
                                        itypes.push(itype);
                                        locals.push(local);
                                        (itypes, locals)
                                    },
                                );

                            add_cache_storage_object_instructions(
                                module,
                                builder,
                                compilation_ctx,
                                &mapped_function.signature.arguments,
                                function_locals,
                            )?;
                            let (flush_cache_fn, _) = storage_flush_cache(module);
                            builder.i32_const(1).call(flush_cache_fn);

                            let external_call_fn = NativeFunction::get_external_call(
                                &function_id.identifier,
                                module,
                                compilation_ctx,
                                &function_information.function_id.module_id,
                                &argument_types,
                                &named_ids_types,
                            )?;

                            // Add as arguments the NamedIds
                            // The first load derefences the IMutRef
                            // The second load loads the NamedId<> struct (first field of the
                            // function)
                            for nid_local in named_ids_locals {
                                builder
                                    .local_get(nid_local)
                                    .load(
                                        compilation_ctx.memory_id,
                                        LoadKind::I32 { atomic: false },
                                        MemArg {
                                            align: 0,
                                            offset: 0,
                                        },
                                    )
                                    .load(
                                        compilation_ctx.memory_id,
                                        LoadKind::I32 { atomic: false },
                                        MemArg {
                                            align: 0,
                                            offset: 0,
                                        },
                                    );
                            }

                            external_call_fn
                        } else {
                            NativeFunction::get(
                                &function_id.identifier,
                                module,
                                compilation_ctx,
                                &function_information.function_id.module_id,
                            )?
                        };

                    builder.call(native_function_id);
                } else {
                    let table_id = function_table.get_table_id();

                    // If the function is in the table we call it directly
                    let f_entry = if let Some(f) = function_table.get_by_function_id(function_id) {
                        f
                    }
                    // Otherwise, we add it to the table and declare it for translating and linking
                    // before calling it
                    else {
                        functions_calls_to_link.push(function_id.clone());
                        function_table.add(module, function_id.clone(), function_information)?
                    };

                    call_indirect(
                        f_entry,
                        &module_data.functions.returns[function_handle_index.into_index()],
                        table_id,
                        builder,
                        module,
                        compilation_ctx,
                    )?;
                };

                // After the call, check if any argument is a mutable reference to a vector
                // and update it if needed.
                if !mut_ref_vec_locals.is_empty() {
                    let update_mut_ref_fn =
                        RuntimeFunction::VecUpdateMutRef.get(module, Some(compilation_ctx))?;
                    for &local in mut_ref_vec_locals.iter() {
                        builder.local_get(local).call(update_mut_ref_fn);
                    }
                }
            }

            // Insert in the stack types the types returned by the function (if any)
            let return_types = &module_data.functions.returns[function_handle_index.0 as usize];
            types_stack.append(return_types);
        }
        // Locals
        Bytecode::StLoc(local_id) => {
            let local = function_locals[*local_id as usize];
            let local_type = &mapped_function.get_local_ir(*local_id as usize);
            // If type is a reference we set the local directly, else we box it.
            if let IntermediateType::IRef(_) | IntermediateType::IMutRef(_) = local_type {
                builder.local_set(local);
            } else {
                local_type.box_local_instructions(module, builder, compilation_ctx, local)?;
            }

            // At the moment of calculating the local types for the function, we can't know if the
            // type the local is holding has some special property.
            // If we find a UID or NamedId, we need to know which struct it belongs to. That
            // information is inside the types stack (filled by the `bytecodes::struct::unpack`
            // function).
            //
            // So, if the local type is a UID, and in the types stack we have a UID holding the
            // parent struct information, we set the `uid_locals` variable with the parent
            // information.
            // That information will be used later when processing the `MoveLoc` bytecode.
            match local_type {
                IntermediateType::IStruct {
                    module_id, index, ..
                } if Uid::is_vm_type(module_id, *index, compilation_ctx)? => {
                    if let Some(IntermediateType::IStruct {
                        module_id: _,
                        index: _,
                        vm_handled_struct:
                            VmHandledStruct::StorageId {
                                parent_module_id,
                                parent_index,
                                instance_types,
                            },
                    }) = &types_stack.iter().last()
                    {
                        uid_locals.insert(
                            *local_id,
                            StorageIdParentInformation {
                                module_id: parent_module_id.clone(),
                                index: *parent_index,
                                instance_types: instance_types.clone(),
                            },
                        );
                    }
                }

                IntermediateType::IGenericStructInstance {
                    module_id, index, ..
                } if NamedId::is_vm_type(module_id, *index, compilation_ctx)? => {
                    if let Some(IntermediateType::IGenericStructInstance {
                        module_id: _,
                        index: _,
                        types: _,
                        vm_handled_struct:
                            VmHandledStruct::StorageId {
                                parent_module_id,
                                parent_index,
                                instance_types,
                            },
                    }) = &types_stack.iter().last()
                    {
                        uid_locals.insert(
                            *local_id,
                            StorageIdParentInformation {
                                module_id: parent_module_id.clone(),
                                index: *parent_index,
                                instance_types: instance_types.clone(),
                            },
                        );
                    }
                }
                _ => (),
            }

            types_stack.pop_expecting(local_type)?;
        }
        Bytecode::MoveLoc(local_id) => {
            // TODO: Find a way to ensure they will not be used again, the Move compiler should do the work for now
            let local = function_locals[*local_id as usize];
            let local_type = mapped_function.get_local_ir(*local_id as usize).clone();
            local_type.move_local_instructions(builder, compilation_ctx, local)?;

            // If we find that the local type we are moving is the UID or NamedId struct, we need
            // to push it in the stacks type with the parent struct information (needed for example,
            // by the UID's delete method).
            //
            // This information can be found inside the `uid_locals` variable, filled by the
            // `StLoc` bytecode
            match &local_type {
                IntermediateType::IStruct {
                    module_id,
                    index,
                    vm_handled_struct: VmHandledStruct::None,
                } if Uid::is_vm_type(module_id, *index, compilation_ctx)? => {
                    if let Some(StorageIdParentInformation {
                        module_id: parent_module_id,
                        index: parent_index,
                        instance_types,
                    }) = uid_locals.get(local_id)
                    {
                        types_stack.push(IntermediateType::IStruct {
                            module_id: module_id.clone(),
                            index: *index,
                            vm_handled_struct: VmHandledStruct::StorageId {
                                parent_module_id: parent_module_id.clone(),
                                parent_index: *parent_index,
                                instance_types: instance_types.clone(),
                            },
                        });
                    } else {
                        types_stack.push(local_type)
                    }
                }
                IntermediateType::IGenericStructInstance {
                    module_id,
                    index,
                    types,
                    vm_handled_struct: VmHandledStruct::None,
                } if NamedId::is_vm_type(module_id, *index, compilation_ctx)? => {
                    if let Some(StorageIdParentInformation {
                        module_id: parent_module_id,
                        index: parent_index,
                        instance_types,
                    }) = uid_locals.get(local_id)
                    {
                        types_stack.push(IntermediateType::IGenericStructInstance {
                            module_id: module_id.clone(),
                            index: *index,
                            types: types.clone(),
                            vm_handled_struct: VmHandledStruct::StorageId {
                                parent_module_id: parent_module_id.clone(),
                                parent_index: *parent_index,
                                instance_types: instance_types.clone(),
                            },
                        });
                    } else {
                        types_stack.push(local_type)
                    }
                }
                _ => types_stack.push(local_type),
            };
        }
        Bytecode::CopyLoc(local_id) => {
            let local = function_locals[*local_id as usize];
            let local_type = mapped_function.get_local_ir(*local_id as usize).clone();
            local_type.copy_local_instructions(
                module,
                builder,
                compilation_ctx,
                module_data,
                local,
            )?;
            types_stack.push(local_type);
        }
        Bytecode::ImmBorrowLoc(local_id) => {
            let local = function_locals[*local_id as usize];
            let local_type = mapped_function.get_local_ir(*local_id as usize).clone();
            builder.local_get(local);
            types_stack.push(IntermediateType::IRef(Box::new(local_type.clone())));
        }
        Bytecode::MutBorrowLoc(local_id) => {
            let local = function_locals[*local_id as usize];
            let local_type = mapped_function.get_local_ir(*local_id as usize).clone();
            builder.local_get(local);
            types_stack.push(IntermediateType::IMutRef(Box::new(local_type.clone())));
        }
        Bytecode::ImmBorrowField(field_id) => {
            let struct_ = module_data.structs.get_by_field_handle_idx(field_id)?;

            // Check if in the types stack we have the correct type
            let t = types_stack.pop()?;

            // In this context, an immutable borrow can coexist with a mutable one, as the Move
            // compiler ensures through static checks that no invalid accesses occur.
            types_stack::match_types!(
                (
                    (IntermediateType::IRef(ref_inner) | IntermediateType::IMutRef(ref_inner)),
                    "reference or mutable reference",
                    t
                ),
                (
                    IntermediateType::IStruct { ref module_id, index, vm_handled_struct },
                    "struct",
                    *ref_inner
                )
            );

            if module_id != &module_data.id || index != struct_.index() {
                return Err(TypesStackError::TypeMismatch {
                    expected: IntermediateType::IStruct {
                        module_id: module_data.id.clone(),
                        index: struct_.index(),
                        vm_handled_struct: VmHandledStruct::None,
                    },
                    found: IntermediateType::IStruct {
                        module_id: module_id.clone(),
                        index,
                        vm_handled_struct,
                    },
                }
                .into());
            }

            let field_type =
                bytecodes::structs::borrow_field(struct_, field_id, builder, compilation_ctx)?;
            let field_type = struct_field_borrow_add_storage_id_parent_information(
                field_type,
                compilation_ctx,
                struct_,
                module_data,
            )?;

            types_stack.push(IntermediateType::IRef(Box::new(field_type)));
        }
        Bytecode::ImmBorrowFieldGeneric(field_id) => {
            let (struct_field_id, instantiation_types) = module_data
                .structs
                .get_instantiated_field_generic_index(field_id)?;

            let instantiation_types = if instantiation_types.iter().any(type_contains_generics) {
                match &types_stack.last() {
                    Some(IntermediateType::IRef(inner))
                    | Some(IntermediateType::IMutRef(inner)) => match &**inner {
                        IntermediateType::IGenericStructInstance { types, .. } => types.clone(),
                        _ => {
                            return Err(TranslationError::ExpectedGenericStructInstance(
                                inner.as_ref().clone(),
                            ));
                        }
                    },
                    _ => return Err(TranslationError::ExpectedGenericStructInstanceNotFound),
                }
            } else {
                instantiation_types.clone()
            };

            let struct_ = if let Ok(struct_) = module_data
                .structs
                .get_struct_instance_by_field_handle_idx(field_id)
            {
                struct_
            } else {
                let generic_stuct = module_data
                    .structs
                    .get_by_field_handle_idx(struct_field_id)?;

                generic_stuct.instantiate(&instantiation_types)
            };

            // Check if in the types stack we have the correct type
            let t = types_stack.pop()?;

            // In this context, an immutable borrow can coexist with a mutable one, as the Move
            // compiler ensures through static checks that no invalid accesses occur.
            types_stack::match_types!(
                (
                    (IntermediateType::IRef(ref_inner) | IntermediateType::IMutRef(ref_inner)),
                    "reference or mutable reference",
                    t
                ),
                (
                    IntermediateType::IGenericStructInstance { ref module_id, index, ref types, vm_handled_struct },
                    "generic struct",
                    *ref_inner
                )
            );
            if module_id != &module_data.id || index != struct_.index() {
                return Err(TypesStackError::TypeMismatch {
                    expected: IntermediateType::IGenericStructInstance {
                        module_id: module_data.id.clone(),
                        index: struct_.index(),
                        types: instantiation_types.clone(),
                        vm_handled_struct: VmHandledStruct::None,
                    },
                    found: IntermediateType::IGenericStructInstance {
                        module_id: module_id.clone(),
                        index,
                        types: types.clone(),
                        vm_handled_struct,
                    },
                }
                .into());
            }

            let field_type = bytecodes::structs::borrow_field(
                &struct_,
                struct_field_id,
                builder,
                compilation_ctx,
            )?;
            let field_type = replace_type_parameters(&field_type, &instantiation_types);
            let field_type = struct_field_borrow_add_storage_id_parent_information(
                field_type,
                compilation_ctx,
                &struct_,
                module_data,
            )?;

            types_stack.push(IntermediateType::IRef(Box::new(field_type)));
        }
        Bytecode::MutBorrowField(field_id) => {
            let struct_ = module_data.structs.get_by_field_handle_idx(field_id)?;

            // Check if in the types stack we have the correct type
            types_stack.pop_expecting(&IntermediateType::IMutRef(Box::new(
                IntermediateType::IStruct {
                    module_id: module_data.id.clone(),
                    index: struct_.index(),
                    vm_handled_struct: VmHandledStruct::None,
                },
            )))?;

            let field_type =
                bytecodes::structs::borrow_field(struct_, field_id, builder, compilation_ctx)?;
            let field_type = struct_field_borrow_add_storage_id_parent_information(
                field_type,
                compilation_ctx,
                struct_,
                module_data,
            )?;

            types_stack.push(IntermediateType::IMutRef(Box::new(field_type)));
        }
        Bytecode::MutBorrowFieldGeneric(field_id) => {
            let (struct_field_id, instantiation_types) = module_data
                .structs
                .get_instantiated_field_generic_index(field_id)?;

            let instantiation_types = if instantiation_types.iter().any(type_contains_generics) {
                match &types_stack.last() {
                    Some(IntermediateType::IRef(inner))
                    | Some(IntermediateType::IMutRef(inner)) => match &**inner {
                        IntermediateType::IGenericStructInstance { types, .. } => types.clone(),
                        _ => {
                            return Err(TranslationError::ExpectedGenericStructInstance(
                                inner.as_ref().clone(),
                            ));
                        }
                    },
                    _ => return Err(TranslationError::ExpectedGenericStructInstanceNotFound),
                }
            } else {
                instantiation_types.clone()
            };

            let struct_ = if let Ok(struct_) = module_data
                .structs
                .get_struct_instance_by_field_handle_idx(field_id)
            {
                struct_
            } else {
                let generic_stuct = module_data
                    .structs
                    .get_by_field_handle_idx(struct_field_id)?;

                generic_stuct.instantiate(&instantiation_types)
            };

            // Check if in the types stack we have the correct type
            types_stack.pop_expecting(&IntermediateType::IMutRef(Box::new(
                IntermediateType::IGenericStructInstance {
                    module_id: module_data.id.clone(),
                    index: struct_.index(),
                    types: instantiation_types.to_vec(),
                    vm_handled_struct: VmHandledStruct::None,
                },
            )))?;

            let field_type = bytecodes::structs::borrow_field(
                &struct_,
                struct_field_id,
                builder,
                compilation_ctx,
            )?;

            let field_type = replace_type_parameters(&field_type, &instantiation_types);
            let field_type = struct_field_borrow_add_storage_id_parent_information(
                field_type,
                compilation_ctx,
                &struct_,
                module_data,
            )?;

            types_stack.push(IntermediateType::IMutRef(Box::new(field_type)));
        }
        Bytecode::VecUnpack(signature_index, length) => {
            let inner =
                bytecodes::vectors::get_inner_type_from_signature(signature_index, module_data)?;

            let inner = if type_contains_generics(&inner) {
                if let Some(caller_type_instances) =
                    &mapped_function.function_id.type_instantiations
                {
                    replace_type_parameters(&inner, caller_type_instances)
                } else {
                    return Err(TranslationError::CouldNotInstantiateGenericTypes);
                }
            } else {
                inner
            };

            types_stack.pop_expecting(&IntermediateType::IVector(Box::new(inner.clone())))?;

            IVector::vec_unpack_instructions(&inner, module, builder, compilation_ctx, *length)?;

            for _ in 0..*length {
                types_stack.push(inner.clone());
            }
        }
        // Vector instructions
        Bytecode::VecImmBorrow(signature_index) => {
            let [t1, t2] = types_stack.pop_n_from_stack()?;

            types_stack::match_types!(
                (IntermediateType::IU64, "u64", t1),
                (IntermediateType::IRef(ref_inner), "vector reference", t2),
                (IntermediateType::IVector(vec_inner), "vector", *ref_inner)
            );

            let expected_vec_inner =
                bytecodes::vectors::get_inner_type_from_signature(signature_index, module_data)?;

            if *vec_inner != expected_vec_inner {
                return Err(TranslationError::TypeMismatch {
                    expected: expected_vec_inner,
                    found: *vec_inner,
                });
            }

            IVector::vec_borrow_instructions(&vec_inner, module, builder, compilation_ctx)?;

            types_stack.push(IntermediateType::IRef(Box::new(*vec_inner)));
        }
        Bytecode::VecMutBorrow(signature_index) => {
            let [t1, t2] = types_stack.pop_n_from_stack()?;

            types_stack::match_types!(
                (IntermediateType::IU64, "u64", t1),
                (
                    IntermediateType::IMutRef(ref_inner),
                    "mutable vector reference",
                    t2
                ),
                (IntermediateType::IVector(vec_inner), "vector", *ref_inner)
            );

            let expected_vec_inner =
                bytecodes::vectors::get_inner_type_from_signature(signature_index, module_data)?;

            if *vec_inner != expected_vec_inner {
                return Err(TranslationError::TypeMismatch {
                    expected: expected_vec_inner,
                    found: *vec_inner,
                });
            }

            IVector::vec_borrow_instructions(&vec_inner, module, builder, compilation_ctx)?;

            types_stack.push(IntermediateType::IMutRef(Box::new(*vec_inner)));
        }
        Bytecode::VecPack(signature_index, num_elements) => {
            // If the inner type is a type parameter, replace it with the last type in the types stack.

            // Example:
            // The function create_foo uses a generic type T that is not known at compilation time. As a result,
            // the VecPack instruction generated for packing the b vector field includes a type parameter instead of a concrete type.
            // When create_foo_u32 is called, it places the specific type onto the types stack. We must substitute the type parameter
            // with the specific type found at the top of the types stack.
            // ```
            // public struct Foo<T: copy> has drop, copy {
            //     a: T,
            //     b: vector<T>,
            // }
            //
            // public fun create_foo<T: copy>(t: T): Foo<T> {
            //     Foo {a: t, b: vector[t, t, t]}
            // }
            //
            // public fun create_foo_u32(t: u32): Foo<u32> {
            //     create_foo(t)
            // }
            // ```

            let inner =
                bytecodes::vectors::get_inner_type_from_signature(signature_index, module_data)?;

            let inner = if type_contains_generics(&inner) {
                if let Some(caller_type_instances) =
                    &mapped_function.function_id.type_instantiations
                {
                    replace_type_parameters(&inner, caller_type_instances)
                } else {
                    return Err(TranslationError::CouldNotInstantiateGenericTypes);
                }
            } else {
                inner
            };

            IVector::vec_pack_instructions(
                &inner,
                module,
                builder,
                compilation_ctx,
                *num_elements as i32,
            )?;

            // Remove the packing values from types stack and check if the types are correct
            let mut n = *num_elements as usize;
            while n > 0 {
                types_stack.pop_expecting(&inner)?;
                n -= 1;
            }

            types_stack.push(IntermediateType::IVector(Box::new(inner)));
        }
        Bytecode::VecPopBack(signature_index) => {
            let ty = types_stack.pop()?;

            types_stack::match_types!(
                (
                    IntermediateType::IMutRef(ref_inner),
                    "mutable vector reference",
                    ty
                ),
                (IntermediateType::IVector(vec_inner), "vector", *ref_inner)
            );

            let expected_vec_inner =
                bytecodes::vectors::get_inner_type_from_signature(signature_index, module_data)?;

            let expected_vec_inner = if let IntermediateType::ITypeParameter(_) = expected_vec_inner
            {
                &*vec_inner
            } else {
                &expected_vec_inner
            };

            if *vec_inner != *expected_vec_inner {
                return Err(TranslationError::TypeMismatch {
                    expected: expected_vec_inner.clone(),
                    found: *vec_inner,
                });
            }

            match *vec_inner {
                IntermediateType::IBool
                | IntermediateType::IU8
                | IntermediateType::IU16
                | IntermediateType::IU32
                | IntermediateType::IU128
                | IntermediateType::IU256
                | IntermediateType::IAddress
                | IntermediateType::ISigner
                | IntermediateType::IStruct { .. }
                | IntermediateType::IGenericStructInstance { .. }
                | IntermediateType::IVector(_)
                | IntermediateType::IEnum { .. }
                | IntermediateType::IGenericEnumInstance { .. } => {
                    let pop_back_f =
                        RuntimeFunction::VecPopBack32.get(module, Some(compilation_ctx))?;
                    builder.call(pop_back_f);
                }
                IntermediateType::IU64 => {
                    let pop_back_f =
                        RuntimeFunction::VecPopBack64.get(module, Some(compilation_ctx))?;
                    builder.call(pop_back_f);
                }
                IntermediateType::ITypeParameter(_)
                | IntermediateType::IRef(_)
                | IntermediateType::IMutRef(_) => {
                    return Err(TranslationError::InvalidOperation {
                        operation: instruction.clone(),
                        operand_type: *vec_inner,
                    });
                }
            }

            types_stack.push(*vec_inner);
        }
        Bytecode::VecPushBack(signature_index) => {
            let [elem_ty, ref_ty] = types_stack.pop_n_from_stack()?;

            types_stack::match_types!(
                (
                    IntermediateType::IMutRef(mut_inner),
                    "mutable vector reference",
                    ref_ty
                ),
                (IntermediateType::IVector(vec_inner), "vector", *mut_inner)
            );

            let expected_elem_type =
                bytecodes::vectors::get_inner_type_from_signature(signature_index, module_data)?;

            let expected_elem_type = if let IntermediateType::ITypeParameter(_) = expected_elem_type
            {
                &*vec_inner
            } else {
                &expected_elem_type
            };

            if *vec_inner != *expected_elem_type {
                return Err(TranslationError::TypeMismatch {
                    expected: expected_elem_type.clone(),
                    found: *vec_inner,
                });
            }

            if &elem_ty != expected_elem_type {
                return Err(TranslationError::TypeMismatch {
                    expected: expected_elem_type.clone(),
                    found: elem_ty,
                });
            }

            IVector::vec_push_back_instructions(
                &elem_ty,
                module,
                builder,
                compilation_ctx,
                module_data,
            )?;
        }
        Bytecode::VecSwap(signature_index) => {
            let [id2_ty, id1_ty, ref_ty] = types_stack.pop_n_from_stack()?;

            types_stack::match_types!(
                (IntermediateType::IU64, "u64", id2_ty),
                (IntermediateType::IU64, "u64", id1_ty),
                (
                    IntermediateType::IMutRef(mut_inner),
                    "mutable vector reference",
                    ref_ty
                ),
                (IntermediateType::IVector(vec_inner), "vector", *mut_inner)
            );

            let expected_vec_inner =
                bytecodes::vectors::get_inner_type_from_signature(signature_index, module_data)?;

            let expected_vec_inner = if type_contains_generics(&expected_vec_inner) {
                if let Some(caller_type_instances) =
                    &mapped_function.function_id.type_instantiations
                {
                    replace_type_parameters(&expected_vec_inner, caller_type_instances)
                } else {
                    return Err(TranslationError::CouldNotInstantiateGenericTypes);
                }
            } else {
                expected_vec_inner
            };

            if *vec_inner != expected_vec_inner {
                return Err(TranslationError::TypeMismatch {
                    expected: expected_vec_inner,
                    found: *vec_inner,
                });
            }

            match *vec_inner {
                IntermediateType::IU64 => {
                    let swap_f = RuntimeFunction::VecSwap64.get(module, Some(compilation_ctx))?;
                    builder.call(swap_f);
                }
                _ => {
                    let swap_f = RuntimeFunction::VecSwap32.get(module, Some(compilation_ctx))?;
                    builder.call(swap_f);
                }
            }
        }
        Bytecode::VecLen(signature_index) => {
            let elem_ir_type =
                bytecodes::vectors::get_inner_type_from_signature(signature_index, module_data)?;

            if let IntermediateType::ITypeParameter(_) = elem_ir_type {
                let ref_ty = types_stack.pop()?;
                types_stack::match_types!(
                    (IntermediateType::IRef(inner), "vector reference", ref_ty),
                    (IntermediateType::IVector(_vec_inner), "vector", *inner)
                );
            } else {
                types_stack.pop_expecting(&IntermediateType::IRef(Box::new(
                    IntermediateType::IVector(Box::new(elem_ir_type)),
                )))?;
            };

            builder
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .unop(UnaryOp::I64ExtendUI32);

            types_stack.push(IntermediateType::IU64);
        }
        Bytecode::ReadRef => {
            let ref_type = types_stack.pop()?;

            types_stack::match_types!((
                (IntermediateType::IRef(inner) | IntermediateType::IMutRef(inner)),
                "IRef or IMutRef",
                ref_type
            ));

            inner.add_read_ref_instructions(builder, module, compilation_ctx, module_data)?;
            types_stack.push(*inner);
        }
        Bytecode::WriteRef => {
            let [iref, value_type] = types_stack.pop_n_from_stack()?;

            types_stack::match_types!((IntermediateType::IMutRef(inner), "IMutRef", iref));

            if *inner == value_type {
                inner.add_write_ref_instructions(module, builder, compilation_ctx)?;
            } else {
                Err(TranslationError::TypeMismatch {
                    expected: *inner,
                    found: value_type,
                })?;
            }
        }
        Bytecode::FreezeRef => {
            let ref_type = types_stack.pop()?;

            types_stack::match_types!((
                IntermediateType::IMutRef(inner),
                "mutable reference",
                ref_type
            ));

            types_stack.push(IntermediateType::IRef(inner.clone()));
        }
        Bytecode::Pop => {
            builder.drop();
            types_stack.pop()?;
        }
        // TODO: ensure this is the last instruction in the move code
        Bytecode::Ret => {
            // If the function is entry and received as an argument an struct that must be saved in
            // storage, we must persist it in case it had some change.
            //
            // We expect that the owner address is just right before the pointer
            if mapped_function.is_entry {
                add_cache_storage_object_instructions(
                    module,
                    builder,
                    compilation_ctx,
                    &mapped_function.signature.arguments,
                    function_locals,
                )?;
            }

            prepare_function_return(
                module,
                builder,
                &mapped_function.signature.returns,
                compilation_ctx,
            )?;

            // We dont pop the return values from the stack, we just check if the types match
            if !types_stack.0.ends_with(&mapped_function.signature.returns) {
                Err(TypesStackError::FunctionReturnTypeMismatch {
                    stack: types_stack.0.last().cloned(),
                    function: mapped_function.signature.returns.clone(),
                })?;
            }
        }
        Bytecode::CastU8 => {
            let original_type = types_stack.pop()?;
            IU8::cast_from(builder, module, original_type, compilation_ctx)?;
            types_stack.push(IntermediateType::IU8);
        }
        Bytecode::CastU16 => {
            let original_type = types_stack.pop()?;
            IU16::cast_from(builder, module, original_type, compilation_ctx)?;
            types_stack.push(IntermediateType::IU16);
        }
        Bytecode::CastU32 => {
            let original_type = types_stack.pop()?;
            IU32::cast_from(builder, module, original_type, compilation_ctx)?;
            types_stack.push(IntermediateType::IU32);
        }
        Bytecode::CastU64 => {
            let original_type = types_stack.pop()?;
            IU64::cast_from(builder, module, original_type, compilation_ctx)?;
            types_stack.push(IntermediateType::IU64);
        }
        Bytecode::CastU128 => {
            let original_type = types_stack.pop()?;
            IU128::cast_from(builder, module, original_type, compilation_ctx)?;
            types_stack.push(IntermediateType::IU128);
        }
        Bytecode::CastU256 => {
            let original_type = types_stack.pop()?;
            IU256::cast_from(builder, module, original_type, compilation_ctx)?;
            types_stack.push(IntermediateType::IU256);
        }
        Bytecode::Add => {
            let [t1, t2] = types_stack.pop_n_from_stack()?;
            if t1 != t2 {
                return Err(TranslationError::OperationTypeMismatch {
                    operand1: t1,
                    operand2: t2,
                    operation: Bytecode::Add,
                });
            }

            match t1 {
                IntermediateType::IU8 => IU8::add(builder, module)?,
                IntermediateType::IU16 => IU16::add(builder, module)?,
                IntermediateType::IU32 => IU32::add(builder, module)?,
                IntermediateType::IU64 => IU64::add(builder, module)?,
                IntermediateType::IU128 => IU128::add(builder, module, compilation_ctx)?,
                IntermediateType::IU256 => IU256::add(builder, module, compilation_ctx)?,
                _ => Err(TranslationError::InvalidBinaryOperation {
                    operation: Bytecode::Add,
                    operands_types: t1,
                })?,
            }

            types_stack.push(t2);
        }
        Bytecode::Sub => {
            let [t1, t2] = types_stack.pop_n_from_stack()?;
            if t1 != t2 {
                return Err(TranslationError::OperationTypeMismatch {
                    operand1: t1,
                    operand2: t2,
                    operation: Bytecode::Sub,
                });
            }

            match t1 {
                IntermediateType::IU8 => IU8::sub(builder, module)?,
                IntermediateType::IU16 => IU16::sub(builder, module)?,
                IntermediateType::IU32 => IU32::sub(builder, module)?,
                IntermediateType::IU64 => IU64::sub(builder, module)?,
                IntermediateType::IU128 => IU128::sub(builder, module, compilation_ctx)?,
                IntermediateType::IU256 => IU256::sub(builder, module, compilation_ctx)?,
                _ => Err(TranslationError::InvalidBinaryOperation {
                    operation: Bytecode::Sub,
                    operands_types: t1,
                })?,
            }

            types_stack.push(t2);
        }
        Bytecode::Mul => {
            let [t1, t2] = types_stack.pop_n_from_stack()?;
            if t1 != t2 {
                return Err(TranslationError::OperationTypeMismatch {
                    operand1: t1,
                    operand2: t2,
                    operation: Bytecode::Mul,
                });
            }

            match t1 {
                IntermediateType::IU8 => IU8::mul(builder, module)?,
                IntermediateType::IU16 => IU16::mul(builder, module)?,
                IntermediateType::IU32 => IU32::mul(builder, module)?,
                IntermediateType::IU64 => IU64::mul(builder, module)?,
                IntermediateType::IU128 => IU128::mul(builder, module, compilation_ctx)?,
                IntermediateType::IU256 => IU256::mul(builder, module, compilation_ctx)?,
                _ => Err(TranslationError::InvalidBinaryOperation {
                    operation: Bytecode::Mul,
                    operands_types: t1,
                })?,
            }

            types_stack.push(t2);
        }
        Bytecode::Div => {
            let [t1, t2] = types_stack.pop_n_from_stack()?;
            if t1 != t2 {
                return Err(TranslationError::OperationTypeMismatch {
                    operand1: t1,
                    operand2: t2,
                    operation: Bytecode::Div,
                });
            }

            match t1 {
                IntermediateType::IU8 => IU8::div(builder),
                IntermediateType::IU16 => IU16::div(builder),
                IntermediateType::IU32 => IU32::div(builder),
                IntermediateType::IU64 => IU64::div(builder),
                IntermediateType::IU128 => IU128::div(builder, module, compilation_ctx)?,
                IntermediateType::IU256 => IU256::div(builder, module, compilation_ctx)?,
                _ => Err(TranslationError::InvalidBinaryOperation {
                    operation: Bytecode::Div,
                    operands_types: t1,
                })?,
            }

            types_stack.push(t2);
        }
        Bytecode::Lt => {
            let [t1, t2] = types_stack.pop_n_from_stack()?;
            if t1 != t2 {
                return Err(TranslationError::OperationTypeMismatch {
                    operand1: t1,
                    operand2: t2,
                    operation: Bytecode::Lt,
                });
            }

            match t1 {
                IntermediateType::IU8 | IntermediateType::IU16 | IntermediateType::IU32 => {
                    builder.binop(BinaryOp::I32LtU);
                }
                IntermediateType::IU64 => {
                    builder.binop(BinaryOp::I64LtU);
                }
                IntermediateType::IU128 => {
                    let less_than_f =
                        RuntimeFunction::LessThan.get(module, Some(compilation_ctx))?;

                    builder.i32_const(IU128::HEAP_SIZE).call(less_than_f);
                }
                IntermediateType::IU256 => {
                    let less_than_f =
                        RuntimeFunction::LessThan.get(module, Some(compilation_ctx))?;

                    builder.i32_const(IU256::HEAP_SIZE).call(less_than_f);
                }
                _ => Err(TranslationError::InvalidBinaryOperation {
                    operation: Bytecode::Lt,
                    operands_types: t1,
                })?,
            }

            types_stack.push(IntermediateType::IBool);
        }
        Bytecode::Le => {
            let [t1, t2] = types_stack.pop_n_from_stack()?;
            if t1 != t2 {
                return Err(TranslationError::OperationTypeMismatch {
                    operand1: t1,
                    operand2: t2,
                    operation: Bytecode::Le,
                });
            }

            match t1 {
                IntermediateType::IU8 | IntermediateType::IU16 | IntermediateType::IU32 => {
                    builder.binop(BinaryOp::I32LeU);
                }
                IntermediateType::IU64 => {
                    builder.binop(BinaryOp::I64LeU);
                }
                // For u128 and u256 instead of doing a <= b, we do !(b < a) == a <= b, this way
                // we can reuse the LessThan function
                IntermediateType::IU128 => {
                    let less_than_f =
                        RuntimeFunction::LessThan.get(module, Some(compilation_ctx))?;

                    // Temp variables to perform the swap
                    let a = module.locals.add(ValType::I32);
                    let b = module.locals.add(ValType::I32);

                    builder
                        .swap(a, b)
                        .i32_const(IU128::HEAP_SIZE)
                        .call(less_than_f)
                        .negate();
                }
                IntermediateType::IU256 => {
                    let less_than_f =
                        RuntimeFunction::LessThan.get(module, Some(compilation_ctx))?;

                    // Temp variables to perform the swap
                    let a = module.locals.add(ValType::I32);
                    let b = module.locals.add(ValType::I32);

                    builder
                        .swap(a, b)
                        .i32_const(IU256::HEAP_SIZE)
                        .call(less_than_f)
                        .negate();
                }
                _ => Err(TranslationError::InvalidBinaryOperation {
                    operation: Bytecode::Le,
                    operands_types: t1,
                })?,
            }

            types_stack.push(IntermediateType::IBool);
        }
        Bytecode::Gt => {
            let [t1, t2] = types_stack.pop_n_from_stack()?;
            if t1 != t2 {
                return Err(TranslationError::OperationTypeMismatch {
                    operand1: t1,
                    operand2: t2,
                    operation: Bytecode::Gt,
                });
            }

            match t1 {
                IntermediateType::IU8 | IntermediateType::IU16 | IntermediateType::IU32 => {
                    builder.binop(BinaryOp::I32GtU);
                }
                IntermediateType::IU64 => {
                    builder.binop(BinaryOp::I64GtU);
                }
                // For u128 and u256 instead of doing a > b, we do b < a, this way we can reuse the
                // LessThan function
                IntermediateType::IU128 => {
                    let less_than_f =
                        RuntimeFunction::LessThan.get(module, Some(compilation_ctx))?;

                    let a = module.locals.add(ValType::I32);
                    let b = module.locals.add(ValType::I32);

                    builder
                        .swap(a, b)
                        .i32_const(IU128::HEAP_SIZE)
                        .call(less_than_f);
                }
                IntermediateType::IU256 => {
                    let less_than_f =
                        RuntimeFunction::LessThan.get(module, Some(compilation_ctx))?;

                    let a = module.locals.add(ValType::I32);
                    let b = module.locals.add(ValType::I32);

                    builder
                        .swap(a, b)
                        .i32_const(IU256::HEAP_SIZE)
                        .call(less_than_f);
                }
                _ => Err(TranslationError::InvalidBinaryOperation {
                    operation: Bytecode::Gt,
                    operands_types: t1,
                })?,
            }

            types_stack.push(IntermediateType::IBool);
        }
        Bytecode::Ge => {
            let [t1, t2] = types_stack.pop_n_from_stack()?;
            if t1 != t2 {
                return Err(TranslationError::OperationTypeMismatch {
                    operand1: t1,
                    operand2: t2,
                    operation: Bytecode::Ge,
                });
            }

            match t1 {
                IntermediateType::IU8 | IntermediateType::IU16 | IntermediateType::IU32 => {
                    builder.binop(BinaryOp::I32GeU);
                }
                IntermediateType::IU64 => {
                    builder.binop(BinaryOp::I64GeU);
                }
                // For u128 and u256 instead of doing a >= b, we do !(a < b) == a >= b, this way we can reuse the
                // LessThan function
                IntermediateType::IU128 => {
                    let less_than_f =
                        RuntimeFunction::LessThan.get(module, Some(compilation_ctx))?;

                    // Compare
                    builder
                        .i32_const(IU128::HEAP_SIZE)
                        .call(less_than_f)
                        .negate();
                }
                IntermediateType::IU256 => {
                    let less_than_f =
                        RuntimeFunction::LessThan.get(module, Some(compilation_ctx))?;

                    builder
                        .i32_const(IU256::HEAP_SIZE)
                        .call(less_than_f)
                        .negate();
                }
                _ => Err(TranslationError::InvalidBinaryOperation {
                    operation: Bytecode::Ge,
                    operands_types: t1,
                })?,
            }

            types_stack.push(IntermediateType::IBool);
        }
        Bytecode::Mod => {
            let [t1, t2] = types_stack.pop_n_from_stack()?;
            if t1 != t2 {
                return Err(TranslationError::OperationTypeMismatch {
                    operand1: t1,
                    operand2: t2,
                    operation: Bytecode::Mod,
                });
            }

            match t1 {
                IntermediateType::IU8 => IU8::remainder(builder),
                IntermediateType::IU16 => IU16::remainder(builder),
                IntermediateType::IU32 => IU32::remainder(builder),
                IntermediateType::IU64 => IU64::remainder(builder),
                IntermediateType::IU128 => IU128::remainder(builder, module, compilation_ctx)?,
                IntermediateType::IU256 => IU256::remainder(builder, module, compilation_ctx)?,
                _ => Err(TranslationError::InvalidBinaryOperation {
                    operation: Bytecode::Mod,
                    operands_types: t1,
                })?,
            }

            types_stack.push(t2);
        }
        Bytecode::Eq => {
            let [t1, t2] = types_stack.pop_n_from_stack()?;
            if t1 != t2 {
                return Err(TranslationError::OperationTypeMismatch {
                    operand1: t1,
                    operand2: t2,
                    operation: Bytecode::Eq,
                });
            }

            t1.load_equality_instructions(module, builder, compilation_ctx, module_data)?;

            types_stack.push(IntermediateType::IBool);
        }
        Bytecode::Neq => {
            let [t1, t2] = types_stack.pop_n_from_stack()?;
            if t1 != t2 {
                return Err(TranslationError::OperationTypeMismatch {
                    operand1: t1,
                    operand2: t2,
                    operation: Bytecode::Neq,
                });
            }

            t1.load_not_equality_instructions(module, builder, compilation_ctx, module_data)?;

            types_stack.push(IntermediateType::IBool);
        }
        Bytecode::Or => {
            types_stack.pop_expecting(&IntermediateType::IBool)?;
            types_stack.pop_expecting(&IntermediateType::IBool)?;
            builder.binop(BinaryOp::I32Or);
            types_stack.push(IntermediateType::IBool);
        }
        Bytecode::And => {
            types_stack.pop_expecting(&IntermediateType::IBool)?;
            types_stack.pop_expecting(&IntermediateType::IBool)?;
            builder.binop(BinaryOp::I32And);
            types_stack.push(IntermediateType::IBool);
        }
        Bytecode::Not => {
            types_stack.pop_expecting(&IntermediateType::IBool)?;
            builder.unop(UnaryOp::I32Eqz);
            types_stack.push(IntermediateType::IBool);
        }
        Bytecode::BitOr => {
            let [t1, t2] = types_stack.pop_n_from_stack()?;
            if t1 != t2 {
                return Err(TranslationError::OperationTypeMismatch {
                    operand1: t1,
                    operand2: t2,
                    operation: Bytecode::BitOr,
                });
            }

            match t1 {
                IntermediateType::IU8 | IntermediateType::IU16 | IntermediateType::IU32 => {
                    builder.binop(BinaryOp::I32Or);
                }
                IntermediateType::IU64 => {
                    builder.binop(BinaryOp::I64Or);
                }
                IntermediateType::IU128 => {
                    IU128::bit_or(builder, module, compilation_ctx);
                }
                IntermediateType::IU256 => {
                    IU256::bit_or(builder, module, compilation_ctx);
                }
                _ => Err(TranslationError::InvalidBinaryOperation {
                    operation: Bytecode::BitOr,
                    operands_types: t1,
                })?,
            }

            types_stack.push(t2);
        }
        Bytecode::BitAnd => {
            let [t1, t2] = types_stack.pop_n_from_stack()?;
            if t1 != t2 {
                return Err(TranslationError::OperationTypeMismatch {
                    operand1: t1,
                    operand2: t2,
                    operation: Bytecode::BitAnd,
                });
            }

            match t1 {
                IntermediateType::IU8 | IntermediateType::IU16 | IntermediateType::IU32 => {
                    builder.binop(BinaryOp::I32And);
                }
                IntermediateType::IU64 => {
                    builder.binop(BinaryOp::I64And);
                }
                IntermediateType::IU128 => {
                    IU128::bit_and(builder, module, compilation_ctx);
                }
                IntermediateType::IU256 => {
                    IU256::bit_and(builder, module, compilation_ctx);
                }
                _ => Err(TranslationError::InvalidBinaryOperation {
                    operation: Bytecode::BitAnd,
                    operands_types: t1,
                })?,
            }

            types_stack.push(t2);
        }
        Bytecode::Abort => {
            // Expect a u64 on the Wasm stack and stash it
            types_stack.pop_expecting(&IntermediateType::IU64)?;

            // Returns a ptr to the encoded error message
            let ptr = build_abort_error_message(builder, module, compilation_ctx)?;

            // Store the ptr at DATA_ABORT_MESSAGE_PTR_OFFSET
            builder
                .i32_const(DATA_ABORT_MESSAGE_PTR_OFFSET)
                .local_get(ptr)
                .store(
                    compilation_ctx.memory_id,
                    StoreKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

            builder.i32_const(1);
            builder.return_();
        }
        Bytecode::Xor => {
            let [t1, t2] = types_stack.pop_n_from_stack()?;
            if t1 != t2 {
                return Err(TranslationError::OperationTypeMismatch {
                    operand1: t1,
                    operand2: t2,
                    operation: Bytecode::BitOr,
                });
            }

            match t1 {
                IntermediateType::IU8 | IntermediateType::IU16 | IntermediateType::IU32 => {
                    builder.binop(BinaryOp::I32Xor);
                }
                IntermediateType::IU64 => {
                    builder.binop(BinaryOp::I64Xor);
                }
                IntermediateType::IU128 => {
                    IU128::bit_xor(builder, module, compilation_ctx);
                }
                IntermediateType::IU256 => {
                    IU256::bit_xor(builder, module, compilation_ctx);
                }
                _ => Err(TranslationError::InvalidBinaryOperation {
                    operation: Bytecode::Xor,
                    operands_types: t2,
                })?,
            }

            types_stack.push(t1);
        }
        Bytecode::Shl => {
            types_stack.pop_expecting(&IntermediateType::IU8)?;
            let t = types_stack.pop()?;
            match t {
                IntermediateType::IU8 => IU8::bit_shift_left(builder, module)?,
                IntermediateType::IU16 => IU16::bit_shift_left(builder, module)?,
                IntermediateType::IU32 => IU32::bit_shift_left(builder, module)?,
                IntermediateType::IU64 => IU64::bit_shift_left(builder, module)?,
                IntermediateType::IU128 => IU128::bit_shift_left(builder, module, compilation_ctx)?,
                IntermediateType::IU256 => IU256::bit_shift_left(builder, module, compilation_ctx)?,
                _ => Err(TranslationError::InvalidOperation {
                    operation: Bytecode::Shl,
                    operand_type: t.clone(),
                })?,
            }
            types_stack.push(t);
        }
        Bytecode::Shr => {
            types_stack.pop_expecting(&IntermediateType::IU8)?;
            let t = types_stack.pop()?;
            match t {
                IntermediateType::IU8 => IU8::bit_shift_right(builder, module)?,
                IntermediateType::IU16 => IU16::bit_shift_right(builder, module)?,
                IntermediateType::IU32 => IU32::bit_shift_right(builder, module)?,
                IntermediateType::IU64 => IU64::bit_shift_right(builder, module)?,
                IntermediateType::IU128 => {
                    IU128::bit_shift_right(builder, module, compilation_ctx)?
                }
                IntermediateType::IU256 => {
                    IU256::bit_shift_right(builder, module, compilation_ctx)?
                }
                _ => Err(TranslationError::InvalidOperation {
                    operation: Bytecode::Shr,
                    operand_type: t.clone(),
                })?,
            }
            types_stack.push(t);
        }
        Bytecode::Pack(struct_definition_index) => {
            let struct_ = module_data
                .structs
                .get_by_struct_definition_idx(struct_definition_index)?;

            // Allocate four bytes that will point to the struct wrapping this UID. It will be
            // filled later in the `bytecodes::structs::pack` function.
            // This information will be used by other operations (such as delete) to locate the struct
            if Uid::is_vm_type(&module_data.id, struct_definition_index.0, compilation_ctx)? {
                builder.i32_const(4).call(compilation_ctx.allocator).drop();
            }

            bytecodes::structs::pack(struct_, module, builder, compilation_ctx, types_stack)?;

            types_stack.push(IntermediateType::IStruct {
                module_id: module_data.id.clone(),
                index: struct_definition_index.0,
                vm_handled_struct: VmHandledStruct::None,
            });
        }
        Bytecode::PackGeneric(struct_definition_index) => {
            let struct_ = module_data
                .structs
                .get_struct_instance_by_struct_definition_idx(struct_definition_index)?;

            let type_instantiations = module_data
                .structs
                .get_generic_struct_types_instances(struct_definition_index)?
                .to_vec();

            // In some situations a struct instantiation in the Move module that contains a generic type
            // parameter. For example:
            // ```
            // public struct Foo<T> {
            //     field: T
            // }
            //
            // public fun create_foo<T>(f: T): Foo<T> {
            //     Foo { field: f }
            // }
            //
            // public fun create_foo_u32(n: u32): Foo<u32> {
            //     create_foo(n)
            // }
            // ```
            //  In `create_foo` the compiler does not have any information about what T could be,
            //  so, when called from `create_foo_u32` it will find a TypeParameter instead of a u32.
            //  The TypeParameter will replaced by the u32 using the caller's type information.
            let (struct_, types) = if type_instantiations.iter().any(type_contains_generics) {
                if let Some(caller_type_instances) =
                    &mapped_function.function_id.type_instantiations
                {
                    let mut instantiations = Vec::new();
                    for field in &type_instantiations {
                        instantiations.push(replace_type_parameters(field, caller_type_instances));
                    }

                    // The purpose of this block is to determine the concrete types with which to instantiate
                    // a generic struct, handling the potentially chained replacement of type parameters.
                    // If some instantiations are still type parameters after one pass, those are filtered out
                    // and the Vec is formed from these. Otherwise, just use the instantiations directly.
                    let type_parameters_instantiations = type_instantiations
                        .iter()
                        .enumerate()
                        .filter_map(|(index, field)| match field {
                            IntermediateType::ITypeParameter(_) => {
                                Some(instantiations[index].clone())
                            }
                            _ => None,
                        })
                        .collect::<Vec<IntermediateType>>();

                    (
                        struct_.instantiate(&type_parameters_instantiations),
                        instantiations,
                    )
                }
                // This should never happen
                else {
                    return Err(TranslationError::CouldNotInstantiateGenericTypes);
                }
            } else {
                let types = module_data
                    .structs
                    .get_generic_struct_types_instances(struct_definition_index)?
                    .to_vec();

                (struct_, types)
            };

            // Allocate four bytes that will point to the struct wrapping this UID. It will be
            // filled later in the `bytecodes::structs::pack` function.
            // This information will be used by other operations (such as delete) to locate the struct
            if NamedId::is_vm_type(&module_data.id, struct_.index(), compilation_ctx)? {
                builder.i32_const(4).call(compilation_ctx.allocator).drop();
            }

            bytecodes::structs::pack(&struct_, module, builder, compilation_ctx, types_stack)?;

            let idx = module_data
                .structs
                .get_generic_struct_idx_by_struct_definition_idx(struct_definition_index);

            types_stack.push(IntermediateType::IGenericStructInstance {
                module_id: module_data.id.clone(),
                index: idx,
                types,
                vm_handled_struct: VmHandledStruct::None,
            });
        }
        Bytecode::Unpack(struct_definition_index) => {
            let itype = types_stack.pop_expecting(&IntermediateType::IStruct {
                module_id: module_data.id.clone(),
                index: struct_definition_index.0,
                vm_handled_struct: VmHandledStruct::None,
            })?;

            let struct_ = module_data
                .structs
                .get_by_struct_definition_idx(struct_definition_index)?;

            bytecodes::structs::unpack(
                struct_,
                &itype,
                module,
                builder,
                compilation_ctx,
                types_stack,
            )?;
        }
        Bytecode::UnpackGeneric(struct_definition_index) => {
            let idx = module_data
                .structs
                .get_generic_struct_idx_by_struct_definition_idx(struct_definition_index);
            let types = module_data
                .structs
                .get_generic_struct_types_instances(struct_definition_index)?;

            let struct_ = module_data
                .structs
                .get_struct_instance_by_struct_definition_idx(struct_definition_index)?;

            let (struct_, types) = if types.iter().any(type_contains_generics) {
                if let Some(caller_type_instances) =
                    &mapped_function.function_id.type_instantiations
                {
                    let mut instantiations = Vec::new();
                    for field in types {
                        instantiations.push(replace_type_parameters(field, caller_type_instances));
                    }

                    (struct_.instantiate(&instantiations), instantiations)
                }
                // This should never happen
                else {
                    return Err(TranslationError::CouldNotInstantiateGenericTypes);
                }
            } else {
                (struct_, types.to_vec())
            };

            let itype = types_stack.pop_expecting(&IntermediateType::IGenericStructInstance {
                module_id: module_data.id.clone(),
                index: idx,
                types,
                vm_handled_struct: VmHandledStruct::None,
            })?;

            bytecodes::structs::unpack(
                &struct_,
                &itype,
                module,
                builder,
                compilation_ctx,
                types_stack,
            )?;
        }
        Bytecode::BrTrue(code_offset) => {
            if let Some(mode) = branches.get(code_offset) {
                if let Some(target) = control_targets.resolve(*mode, *code_offset)? {
                    builder.br_if(target);
                } else {
                    return Err(TranslationError::BranchTargetNotFound(*code_offset));
                }
            }
        }
        Bytecode::BrFalse(code_offset) => {
            if let Some(mode) = branches.get(code_offset) {
                if let Some(target) = control_targets.resolve(*mode, *code_offset)? {
                    // flip the boolean (Move’s BrFalse consumes the bool)
                    builder.unop(UnaryOp::I32Eqz);
                    builder.br_if(target);
                } else {
                    return Err(TranslationError::BranchTargetNotFound(*code_offset));
                }
            }
        }
        Bytecode::Branch(code_offset) => {
            if let Some(mode) = branches.get(code_offset) {
                if let Some(target) = control_targets.resolve(*mode, *code_offset)? {
                    builder.br(target);
                } else {
                    return Err(TranslationError::BranchTargetNotFound(*code_offset));
                }
            }
        }
        Bytecode::Nop => {
            // Just do nothing
        }
        //**
        // Enums
        //**
        Bytecode::PackVariant(index) => {
            let enum_ = module_data.enums.get_enum_by_variant_handle_idx(index)?;
            let variant_index = module_data
                .enums
                .get_variant_position_by_variant_handle_idx(index)?;

            bytecodes::enums::pack_variant(
                enum_,
                variant_index,
                module,
                builder,
                compilation_ctx,
                types_stack,
            )?;

            types_stack.push(IntermediateType::IEnum {
                module_id: module_data.id.clone(),
                index: enum_.index,
            });
        }
        Bytecode::PackVariantGeneric(index) => {
            let enum_ = &module_data
                .enums
                .get_enum_instance_by_variant_instantiation_handle_idx(index)?;

            let variant_index = module_data
                .enums
                .get_variant_position_by_variant_instantiation_handle_idx(index)?;

            let type_instantiations = module_data.enums.get_enum_instance_types(index)?;

            let (enum_, types) = if type_instantiations.iter().any(type_contains_generics) {
                if let Some(caller_type_instances) =
                    &mapped_function.function_id.type_instantiations
                {
                    let mut instantiations = Vec::new();
                    for field in type_instantiations {
                        instantiations.push(replace_type_parameters(field, caller_type_instances));
                    }

                    let type_parameters_instantiations = type_instantiations
                        .iter()
                        .enumerate()
                        .filter_map(|(index, field)| match field {
                            IntermediateType::ITypeParameter(_) => {
                                Some(instantiations[index].clone())
                            }
                            _ => None,
                        })
                        .collect::<Vec<IntermediateType>>();

                    (
                        &enum_.instantiate(&type_parameters_instantiations),
                        instantiations,
                    )
                }
                // This should never happen
                else {
                    return Err(TranslationError::CouldNotInstantiateGenericTypes);
                }
            } else {
                let types = module_data.enums.get_enum_instance_types(index)?.to_vec();

                (enum_, types)
            };

            bytecodes::enums::pack_variant(
                enum_,
                variant_index,
                module,
                builder,
                compilation_ctx,
                types_stack,
            )?;

            types_stack.push(IntermediateType::IGenericEnumInstance {
                module_id: module_data.id.clone(),
                index: enum_.index,
                types,
            });
        }
        Bytecode::UnpackVariant(index) => {
            let enum_ = module_data.enums.get_enum_by_variant_handle_idx(index)?;
            let variant_index = module_data
                .enums
                .get_variant_position_by_variant_handle_idx(index)?;

            let itype = types_stack.pop_expecting(&IntermediateType::IEnum {
                module_id: module_data.id.clone(),
                index: enum_.index,
            })?;

            bytecodes::enums::unpack_variant(
                enum_,
                variant_index,
                module,
                builder,
                compilation_ctx,
                types_stack,
                &itype,
            )?;
        }
        Bytecode::UnpackVariantGeneric(index) => {
            let enum_ = &module_data
                .enums
                .get_enum_instance_by_variant_instantiation_handle_idx(index)?;
            let variant_index = module_data
                .enums
                .get_variant_position_by_variant_instantiation_handle_idx(index)?;
            let type_instantiations = module_data.enums.get_enum_instance_types(index)?.to_vec();

            let (enum_, types) = if type_instantiations.iter().any(type_contains_generics) {
                if let Some(caller_type_instances) =
                    &mapped_function.function_id.type_instantiations
                {
                    let mut instantiations = Vec::new();
                    for field in &type_instantiations {
                        instantiations.push(replace_type_parameters(field, caller_type_instances));
                    }

                    (&enum_.instantiate(&instantiations), instantiations)
                }
                // This should never happen
                else {
                    return Err(TranslationError::CouldNotInstantiateGenericTypes);
                }
            } else {
                (enum_, type_instantiations.to_vec())
            };

            let itype = types_stack.pop_expecting(&IntermediateType::IGenericEnumInstance {
                module_id: module_data.id.clone(),
                index: enum_.index,
                types,
            })?;

            bytecodes::enums::unpack_variant(
                enum_,
                variant_index,
                module,
                builder,
                compilation_ctx,
                types_stack,
                &itype,
            )?;
        }
        Bytecode::UnpackVariantImmRef(index) | Bytecode::UnpackVariantMutRef(index) => {
            let enum_ = module_data.enums.get_enum_by_variant_handle_idx(index)?;
            let variant_index = module_data
                .enums
                .get_variant_position_by_variant_handle_idx(index)?;

            let is_mut_ref = matches!(instruction, Bytecode::UnpackVariantMutRef(_));
            let enum_type = IntermediateType::IEnum {
                module_id: module_data.id.clone(),
                index: enum_.index,
            };
            let expected_type = if is_mut_ref {
                IntermediateType::IMutRef(Box::new(enum_type))
            } else {
                IntermediateType::IRef(Box::new(enum_type))
            };
            types_stack.pop_expecting(&expected_type)?;

            bytecodes::enums::unpack_variant_ref(
                enum_,
                variant_index,
                module,
                builder,
                compilation_ctx,
                types_stack,
                is_mut_ref,
            )?;
        }
        Bytecode::UnpackVariantGenericImmRef(index)
        | Bytecode::UnpackVariantGenericMutRef(index) => {
            let enum_ = &module_data
                .enums
                .get_enum_instance_by_variant_instantiation_handle_idx(index)?;
            let variant_index = module_data
                .enums
                .get_variant_position_by_variant_instantiation_handle_idx(index)?;

            let type_instantiations = module_data.enums.get_enum_instance_types(index)?.to_vec();

            let (enum_, types) = if type_instantiations.iter().any(type_contains_generics) {
                if let Some(caller_type_instances) =
                    &mapped_function.function_id.type_instantiations
                {
                    let mut instantiations = Vec::new();
                    for field in &type_instantiations {
                        instantiations.push(replace_type_parameters(field, caller_type_instances));
                    }

                    (&enum_.instantiate(&instantiations), instantiations)
                }
                // This should never happen
                else {
                    return Err(TranslationError::CouldNotInstantiateGenericTypes);
                }
            } else {
                (enum_, type_instantiations.to_vec())
            };

            let is_mut_ref = matches!(instruction, Bytecode::UnpackVariantGenericMutRef(_));
            let generic_enum_type = IntermediateType::IGenericEnumInstance {
                module_id: module_data.id.clone(),
                index: enum_.index,
                types,
            };
            let expected_type = if is_mut_ref {
                IntermediateType::IMutRef(Box::new(generic_enum_type))
            } else {
                IntermediateType::IRef(Box::new(generic_enum_type))
            };
            types_stack.pop_expecting(&expected_type)?;

            bytecodes::enums::unpack_variant_ref(
                enum_,
                variant_index,
                module,
                builder,
                compilation_ctx,
                types_stack,
                is_mut_ref,
            )?;
        }
        Bytecode::VariantSwitch(jump_table_index) => {
            // The match subject (&IEnum) is on top of the stack
            // The first load is to load the enum pointer from the reference
            // The second load is to load the variant index from the enum pointer
            builder
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

            // Set the jump table for the current VariantSwitch
            **jump_table = Some(mapped_function.jump_tables[jump_table_index.0 as usize].clone());
        }
        b => Err(TranslationError::UnsupportedOperation {
            operation: b.clone(),
        })?,
    }

    Ok(functions_calls_to_link)
}

fn call_indirect(
    function_entry: &TableEntry,
    function_returns: &[IntermediateType],
    wasm_table_id: TableId,
    builder: &mut InstrSeqBuilder,
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<(), TranslationError> {
    builder
        .i32_const(function_entry.index)
        .call_indirect(function_entry.type_id, wasm_table_id);

    add_unpack_function_return_values_instructions(
        builder,
        module,
        function_returns,
        compilation_ctx.memory_id,
    )?;

    Ok(())
}

fn process_fn_local_variables(
    function_information: &MappedFunction,
    module: &mut Module,
) -> Result<(Vec<LocalId>, Vec<LocalId>), TranslationError> {
    let wasm_arg_types = function_information.signature.get_argument_wasm_types()?;
    let wasm_ret_types = function_information.signature.get_return_wasm_types();
    if wasm_ret_types.len() > 1 {
        return Err(TranslationError::MultipleWasmReturnValues(
            wasm_ret_types.len(),
        ));
    }

    // WASM locals for arguments
    let wasm_arg_locals: Vec<LocalId> = wasm_arg_types
        .iter()
        .map(|ty| module.locals.add(*ty))
        .collect();

    let wasm_declared_locals = function_information
        .locals
        .iter()
        .map(|ty| {
            match ty {
                IntermediateType::IU64 => Ok(ValType::I32), // to store pointer instead of i64
                other => ValType::try_from(other),
            }
        })
        .collect::<Result<Vec<ValType>, IntermediateTypeError>>()?
        .into_iter()
        .map(|ty| module.locals.add(ty))
        .collect();

    Ok((wasm_arg_locals, wasm_declared_locals))
}

/// Converts value-based function arguments into heap-allocated pointers.
///
/// For each value-type argument (like u64, u32, etc.), this stores the value in linear memory
/// and updates the local to hold a pointer to that memory instead. This allows treating all
/// arguments as pointers in later code.
pub fn box_args(
    builder: &mut InstrSeqBuilder,
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    function_locals: &mut [LocalId],
    function_information: &MappedFunction,
) -> Result<(), TranslationError> {
    // Store the changes we need to make
    let mut updates = Vec::new();

    // Iterate over the mapped function arguments
    for (local, ty) in function_locals
        .iter()
        .zip(&function_information.signature.arguments)
    {
        builder.local_get(*local);
        match ty {
            IntermediateType::IU64 => {
                let outer_ptr = module.locals.add(ValType::I32);
                ty.box_local_instructions(module, builder, compilation_ctx, outer_ptr)?;

                if let Some(index) = function_locals.iter().position(|&id| id == *local) {
                    updates.push((index, outer_ptr));
                } else {
                    return Err(TranslationError::LocalNotFound(*local));
                }
            }
            _ => {
                ty.box_local_instructions(module, builder, compilation_ctx, *local)?;
            }
        }
    }

    for (index, pointer) in updates {
        function_locals[index] = pointer;
    }

    Ok(())
}

/// This function saves into the storage cache all the changes made to the storage objects of the
/// executing function.
/// This is used in two situations:
/// - at the end of a an entry function.
/// - right before a delegate call.
///
/// This function does not flush the cache. That must be done manually depending on the context.
fn add_cache_storage_object_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    function_argumets: &[IntermediateType],
    function_locals: &[LocalId],
) -> Result<(), TranslationError> {
    let object_to_cache = function_argumets
        .iter()
        .enumerate()
        .filter_map(|(arg_index, fn_arg)| {
            let (itype, struct_) = match fn_arg {
                IntermediateType::IMutRef(inner) => (
                    &**inner,
                    compilation_ctx.get_struct_by_intermediate_type(inner),
                ),
                t => (fn_arg, compilation_ctx.get_struct_by_intermediate_type(t)),
            };

            if let Ok(struct_) = struct_ {
                if struct_.has_key {
                    Some((itype, function_locals[arg_index]))
                } else {
                    None
                }
            } else {
                None
            }
        });

    for (itype, wasm_local_var) in object_to_cache {
        let cache_storage_object_changes_fn = RuntimeFunction::CacheStorageObjectChanges
            .get_generic(module, compilation_ctx, &[itype])?;

        builder
            .local_get(wasm_local_var)
            .call(cache_storage_object_changes_fn);
    }

    Ok(())
}

/// This function checks if the field we are borrowing from struct is a UID or NamedId. If we are
/// borrowing an ID, then we add the information about the parent struct that contains such ID.
///
/// This information is used in contexts where we only have the UID and we need to know from which
/// struct is coming from
fn struct_field_borrow_add_storage_id_parent_information(
    field_type: IntermediateType,
    compilation_ctx: &CompilationContext,
    struct_: &IStruct,
    module_data: &ModuleData,
) -> Result<IntermediateType, TranslationError> {
    // If we borrow a UID, we fill the typestack with the parent struct information
    match &field_type {
        IntermediateType::IStruct {
            module_id,
            index,
            vm_handled_struct: VmHandledStruct::None,
        } if Uid::is_vm_type(module_id, *index, compilation_ctx)? => {
            Ok(IntermediateType::IStruct {
                module_id: module_id.clone(),
                index: *index,
                vm_handled_struct: VmHandledStruct::StorageId {
                    parent_module_id: module_data.id.clone(),
                    parent_index: struct_.index(),
                    instance_types: None,
                },
            })
        }
        IntermediateType::IGenericStructInstance {
            module_id,
            index,
            types,
            vm_handled_struct: VmHandledStruct::None,
        } if NamedId::is_vm_type(module_id, *index, compilation_ctx)? => {
            Ok(IntermediateType::IGenericStructInstance {
                module_id: module_id.clone(),
                index: *index,
                types: types.clone(),
                vm_handled_struct: VmHandledStruct::StorageId {
                    parent_module_id: module_data.id.clone(),
                    parent_index: struct_.index(),
                    instance_types: Some(types.clone()),
                },
            })
        }
        _ => Ok(field_type),
    }
}

// Along with the argument types, we need to collect the NamedId's that
// appear in this function, since in the external call's signature they
// are not present, an we will not be able to update them if they
// change.
fn get_storage_structs_with_named_ids(
    mapped_function: &MappedFunction,
    compilation_ctx: &CompilationContext,
    function_locals: &[LocalId],
) -> Result<Vec<(IntermediateType, LocalId)>, TranslationError> {
    let mut result = Vec::new();
    for (arg_index, fn_arg) in mapped_function.signature.arguments.iter().enumerate() {
        let (itype, struct_) = match fn_arg {
            // We look for both mutable and immutable references of NamedIds, because the collected
            // ones are used to generate the code for the delegate call function used (when using
            // `NativeFunction::get_external_call`.
            //
            // If we have two functions that use the same delegate call, for example:
            //
            // ```
            //   public struct Counter has key {
            //      id: NamedId<COUNTER_>,
            //      value: u64,
            //   }
            //
            //   entry fun increment(counter: &Counter) {
            //       let delegated_counter = dci::new(
            //           contract_calls::new(counter.contract_address)
            //               .delegate()
            //       );
            //       let res = delegated_counter.increment();
            //       assert!(res.succeded(), 33);
            //   }
            //
            //   entry fun increment2(counter: &mut Counter) {
            //       let delegated_counter = dci::new(
            //           contract_calls::new(counter.contract_address)
            //               .delegate()
            //       );
            //       let res = delegated_counter.increment();
            //       assert!(res.succeded(), 33);
            //   }
            // ```
            //
            // and if we only collect mutable ones, the external `increment` function will be
            // generated with the wrong number of arguments.
            //
            // Another reason to include immutable references is that we don't know if the
            // delegated call will change the storage or not. That is independent of the mut
            // declaration of the NamedId
            IntermediateType::IMutRef(inner) | IntermediateType::IRef(inner) => (
                &**inner,
                compilation_ctx.get_struct_by_intermediate_type(inner),
            ),
            t => (fn_arg, compilation_ctx.get_struct_by_intermediate_type(t)),
        };

        let struct_ = match struct_ {
            Ok(s) => s,
            Err(_) => continue,
        };

        let instance_types = if let IntermediateType::IGenericStructInstance { types, .. } = itype {
            Some(types.clone())
        } else {
            None
        };

        let parent_module_id = match itype {
            IntermediateType::IStruct { module_id, .. } => module_id,
            IntermediateType::IGenericStructInstance { module_id, .. } => module_id,
            _ => continue,
        };

        if struct_.has_key {
            match struct_.fields.first() {
                Some(IntermediateType::IGenericStructInstance {
                    module_id,
                    index,
                    types,
                    ..
                }) if NamedId::is_vm_type(module_id, *index, compilation_ctx)? => {
                    result.push((
                        IntermediateType::IGenericStructInstance {
                            module_id: module_id.clone(),
                            index: *index,
                            types: types.clone(),
                            vm_handled_struct: VmHandledStruct::StorageId {
                                parent_module_id: parent_module_id.clone(),
                                parent_index: struct_.index(),
                                instance_types,
                            },
                        },
                        function_locals[arg_index],
                    ));
                }
                _ => continue,
            }
        } else {
            continue;
        }
    }

    Ok(result)
}

/// Trnaslates a function to WASM and links it to the WASM module
///
/// It also recursively translates and links all the functions called by this function
pub(crate) fn translate_and_link_functions(
    function_id: &FunctionId,
    function_table: &mut FunctionTable,
    function_definitions: &GlobalFunctionTable,
    module: &mut walrus::Module,
    compilation_ctx: &CompilationContext,
    dynamic_fields_global_variables: &mut Vec<(GlobalId, IntermediateType)>,
) -> Result<(), TranslationError> {
    // Obtain the function information and module's data
    let (function_information, module_data) = if let Some(fi) = compilation_ctx
        .root_module_data
        .functions
        .information
        .iter()
        .find(|f| {
            f.function_id.module_id == function_id.module_id
                && f.function_id.identifier == function_id.identifier
        }) {
        (fi, compilation_ctx.root_module_data)
    } else {
        let module_data = compilation_ctx.get_module_data_by_id(&function_id.module_id)?;

        let fi = module_data
            .functions
            .get_information_by_identifier(&function_id.identifier)?;

        (fi, module_data)
    };

    // If the function is generic, we instantiate the concrete types so we can translate it
    let function_information = if function_information.is_generic {
        &function_information.instantiate(function_id.type_instantiations.as_ref().ok_or(
            TranslationError::GenericFunctionNoTypeInstantiations(
                function_id.module_id.clone(),
                function_id.identifier.clone(),
            ),
        )?)
    } else {
        function_information
    };

    // Process function defined in this module
    // First we check if there is already an entry for this function
    if let Some(table_entry) = function_table.get_by_function_id(&function_information.function_id)
    {
        // If it has asigned a wasm function id means that we already translated it, so we skip
        // it
        if table_entry.wasm_function_id.is_some() {
            return Ok(());
        }
    }
    // If it is not present, we add an entry for it
    else {
        function_table.add(module, function_id.clone(), function_information)?;
    }

    let function_definition = function_definitions
        .get(&function_id.get_generic_fn_id())
        .ok_or_else(|| TranslationError::FunctionDefinitionNotFound(function_id.clone()))?;

    // If the function contains code we translate it
    // If it does not it means is a native function, we do nothing, it is linked and called
    // directly in the translation function
    if let Some(move_bytecode) = function_definition.code.as_ref() {
        let (wasm_function_id, functions_to_link) = translate_function(
            module,
            compilation_ctx,
            module_data,
            function_table,
            function_information,
            move_bytecode,
            dynamic_fields_global_variables,
        )?;

        function_table.add_to_wasm_table(module, function_id, wasm_function_id)?;

        // Recursively translate and link functions called by this function
        for function_id in &functions_to_link {
            translate_and_link_functions(
                function_id,
                function_table,
                function_definitions,
                module,
                compilation_ctx,
                dynamic_fields_global_variables,
            )?;
        }
    }

    Ok(())
}
