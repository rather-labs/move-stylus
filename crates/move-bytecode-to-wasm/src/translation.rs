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

use crate::{
    CompilationContext, compilation_context::ModuleData, native_functions::NativeFunction,
    runtime::RuntimeFunction, wasm_builder_extensions::WasmBuilderExtension,
};
use anyhow::Result;
use flow::Flow;
use functions::{
    MappedFunction, add_unpack_function_return_values_instructions, prepare_function_return,
};
use intermediate_types::{
    IntermediateType,
    heap_integers::{IU128, IU256},
    simple_integers::{IU8, IU16, IU32, IU64},
    vector::IVector,
};
use move_binary_format::{
    file_format::{Bytecode, CodeUnit},
    internals::ModuleIndex,
};
use relooper::BranchMode;
use std::collections::{HashMap, HashSet};
use table::{FunctionId, FunctionTable, TableEntry};
use types_stack::TypesStack;
use walrus::TableId;
use walrus::{
    FunctionBuilder, FunctionId as WasmFunctionId, InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, Block, IfElse, InstrSeqId, InstrSeqType, LoadKind, MemArg, UnaryOp},
};

/// This struct maps the relooper asigned labels to the actual walrus instruction sequence IDs.
/// It is used to translate the branching instructions: Branch, BrFalse, BrTrue
struct BranchTargets {
    loop_continue: HashMap<u16, InstrSeqId>,
    loop_break: HashMap<u16, InstrSeqId>,
    merged_branch: HashMap<u16, InstrSeqId>,
}

impl BranchTargets {
    fn new() -> Self {
        Self {
            loop_continue: HashMap::new(),
            loop_break: HashMap::new(),
            merged_branch: HashMap::new(),
        }
    }

    fn get_target(&self, branch_mode: &BranchMode, code_offset: &u16) -> Option<&InstrSeqId> {
        match branch_mode {
            BranchMode::LoopContinue(id) | BranchMode::LoopContinueIntoMulti(id) => {
                self.loop_continue.get(id)
            }
            BranchMode::LoopBreak(id) | BranchMode::LoopBreakIntoMulti(id) => {
                self.loop_break.get(id)
            }
            BranchMode::MergedBranch | BranchMode::MergedBranchIntoMulti => {
                self.merged_branch.get(code_offset)
            }
            _ => {
                panic!("Unsupported branch mode: {:?}", branch_mode);
            }
        }
    }
}

/// This is used to pass around the context of the translation process. Also clippy complains about too many arguments in translate_instruction.
struct TranslateFlowContext<'a> {
    compilation_ctx: &'a CompilationContext<'a>,
    module_data: &'a ModuleData,
    types_stack: &'a mut TypesStack,
    function_information: &'a MappedFunction,
    function_table: &'a mut FunctionTable,
    function_locals: &'a Vec<LocalId>,
    branch_targets: &'a mut BranchTargets,
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
) -> Result<(WasmFunctionId, HashSet<FunctionId>)> {
    anyhow::ensure!(
        move_bytecode.jump_tables.is_empty(),
        "Jump tables are not supported yet"
    );

    let params = function_information.signature.get_argument_wasm_types();
    let results = function_information.signature.get_return_wasm_types();
    let mut function = FunctionBuilder::new(&mut module.types, &params, &results);
    let mut builder = function.func_body();

    let (arguments, locals) = process_fn_local_variables(function_information, module);

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
    );

    // let entry = function_table
    //     .get_by_function_id(&function_information.function_id)
    //     .ok_or(anyhow::anyhow!("index {} not found in function table", function_information.function_id))?;

    // let code_unit = &entry.get_move_code_unit().unwrap();

    let flow = Flow::new(move_bytecode, function_information);

    let mut branch_targets = BranchTargets::new();
    let mut types_stack = TypesStack::new();
    let mut functions_to_link = HashSet::new();

    let mut ctx = TranslateFlowContext {
        compilation_ctx,
        module_data,
        function_table,
        function_information,
        function_locals: &function_locals,
        types_stack: &mut types_stack,
        branch_targets: &mut branch_targets,
    };

    translate_flow(
        &mut ctx,
        &mut builder,
        module,
        &flow,
        &mut functions_to_link,
    );

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
) {
    match flow {
        Flow::Simple {
            instructions,
            stack,
            branches,
            immediate,
            next,
            ..
        } => {
            let ty = InstrSeqType::new(&mut module.types, &[], stack);
            builder.block(ty, |block| {
                // When we encounter a MergedBranch target for the first time, we map it to the block's ID (block.id()) that wraps the current simple flow.
                // This mapping determines where to jump when a branch instruction targets that code_offset.
                // Essentially, it allows skipping the current simple block and continuing to the next one.
                for (target_label, branch_mode) in branches {
                    if let BranchMode::MergedBranch | BranchMode::MergedBranchIntoMulti =
                        branch_mode
                    {
                        ctx.branch_targets
                            .merged_branch
                            .insert(*target_label, block.id());
                    }
                }

                // First translate the instuctions associated with the simple flow itself
                for instruction in instructions {
                    let mut fns_to_link = translate_instruction(
                        instruction,
                        ctx.compilation_ctx,
                        ctx.module_data,
                        block,
                        ctx.function_information,
                        module,
                        ctx.function_table,
                        ctx.types_stack,
                        ctx.function_locals,
                        branches,
                        ctx.branch_targets,
                    )
                    .unwrap();
                    functions_to_link.extend(fns_to_link.drain(..));
                }
                // Then translate instructions of the immediate block, inside the current block
                translate_flow(ctx, block, module, immediate, functions_to_link);
            });
            // Then translate instructions of the next block, but outside the wrapping block
            translate_flow(ctx, builder, module, next, functions_to_link);
        }
        Flow::Loop {
            stack,
            loop_id,
            inner,
            next,
            ..
        } => {
            let ty = InstrSeqType::new(&mut module.types, &[], stack);
            // We wrap the loop in a block so we have a "landing spot" if we need to break out of it
            // (in case we encounter a BranchMode::LoopBreak).
            builder.block(ty, |block| {
                // Map the Relooper's loop id to the Walrus instruction sequence ID
                // Walrus needs the specific InstrSeqId where to branch to.
                ctx.branch_targets.loop_break.insert(*loop_id, block.id());

                block.loop_(ty, |loop_| {
                    // Map the loop_id to the actual loop instruction, so `continue` knows where to jump.
                    ctx.branch_targets
                        .loop_continue
                        .insert(*loop_id, loop_.id());

                    // Translate the loop body (inner) inside the loop block.
                    translate_flow(ctx, loop_, module, inner, functions_to_link);
                });
            });

            // Translate the next flow outside the wrapping block.
            translate_flow(ctx, builder, module, next, functions_to_link);
        }
        Flow::IfElse {
            then_body,
            else_body,
            ..
        } => {
            let then_stack = then_body.get_stack();
            let else_stack = else_body.get_stack();

            if then_stack == else_stack {
                let ty = InstrSeqType::new(&mut module.types, &[], &then_stack);
                let then_id = {
                    let mut then_seq = builder.dangling_instr_seq(ty);
                    translate_flow(ctx, &mut then_seq, module, then_body, functions_to_link);
                    then_seq.id()
                };

                let else_id = {
                    let mut else_seq = builder.dangling_instr_seq(ty);
                    translate_flow(ctx, &mut else_seq, module, else_body, functions_to_link);
                    else_seq.id()
                };

                builder.if_else(
                    ty,
                    |then| {
                        then.instr(Block { seq: then_id });
                    },
                    |else_| {
                        else_.instr(Block { seq: else_id });
                    },
                );
            } else if then_stack.is_empty() {
                // If the `then` arm leaves nothing on the stack but the `else` arm does,
                // we place the `else` arm after the if/else block.
                // This situation typically occurs when the arm leaving values on the stack
                // represents a function return, while the other arm simply reloops.
                // The else arm can be safely placed outside the if/else block because it must always be reached,
                // otherwise the block wouldnt have a defined result.
                let phantom_seq = builder.dangling_instr_seq(None);
                let phantom_seq_id = phantom_seq.id();

                let then_id = {
                    let mut then_seq = builder.dangling_instr_seq(None);
                    translate_flow(ctx, &mut then_seq, module, then_body, functions_to_link);
                    then_seq.id()
                };

                builder.instr(IfElse {
                    consequent: then_id,
                    alternative: phantom_seq_id,
                });

                translate_flow(ctx, builder, module, else_body, functions_to_link);
            } else if else_stack.is_empty() {
                // Similar to the above scenario.
                let phantom_seq = builder.dangling_instr_seq(None);
                let phantom_seq_id = phantom_seq.id();

                let else_id = {
                    let mut else_seq = builder.dangling_instr_seq(None);
                    translate_flow(ctx, &mut else_seq, module, else_body, functions_to_link);
                    else_seq.id()
                };

                builder.unop(UnaryOp::I32Eqz);
                builder.instr(IfElse {
                    consequent: else_id,
                    alternative: phantom_seq_id,
                });

                translate_flow(ctx, builder, module, then_body, functions_to_link);
            } else {
                // If both arms leave values on the stack but with different types, we panic.
                // In this scenario, the block wouldnt have a well-defined result type.
                panic!(
                    "Error: Mismatched types on the stack from Then and Else branches, and neither is empty."
                );
            }
        }
        Flow::Empty => (),
    }
}

#[allow(clippy::too_many_arguments)]
fn translate_instruction(
    instruction: &Bytecode,
    compilation_ctx: &CompilationContext,
    module_data: &ModuleData,
    builder: &mut InstrSeqBuilder,
    mapped_function: &MappedFunction,
    module: &mut Module,
    function_table: &mut FunctionTable,
    types_stack: &mut TypesStack,
    function_locals: &[LocalId],
    branches: &HashMap<u16, BranchMode>,
    branch_targets: &BranchTargets,
) -> Result<Vec<FunctionId>, TranslationError> {
    let mut functions_calls_to_link = Vec::new();

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

            constant_type.load_constant_instructions(module, builder, &mut data, compilation_ctx);

            types_stack.push(constant_type);
            assert!(
                data.next().is_none(),
                "Constant data not consumed: {:?}",
                data
            );
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
            );
            types_stack.push(IntermediateType::IU128);
        }
        Bytecode::LdU256(literal) => {
            bytecodes::constants::load_literal_heap_type_to_memory(
                module,
                builder,
                compilation_ctx,
                &literal.to_le_bytes(),
            );
            types_stack.push(IntermediateType::IU256);
        }
        // Function calls
        Bytecode::Call(function_handle_index) => {
            // Consume from the types stack the arguments that will be used by the function call
            let arguments = &module_data.functions.arguments[function_handle_index.into_index()];

            for argument in arguments.iter().rev() {
                types_stack.pop_expecting(argument)?;

                if let IntermediateType::IMutRef(_) | IntermediateType::IRef(_) = argument {
                    builder.load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );
                }
            }

            let function_id = &module_data.functions.calls[function_handle_index.into_index()];

            // If the function is in the table we call it directly
            if let Some(f) = function_table.get_by_function_id(function_id) {
                call_indirect(
                    f,
                    &module_data.functions.returns[function_handle_index.into_index()],
                    function_table.get_table_id(),
                    builder,
                    module,
                    compilation_ctx,
                );
            }
            // Otherwise
            // If the function is not native, we add it to the table and declare it for translating
            // and linking
            // If the function IS native, we link it and call it directly
            else {
                let function_information = if let Some(fi) = module_data
                    .functions
                    .information
                    .get(function_handle_index.into_index())
                {
                    fi
                } else {
                    let dependency_data = compilation_ctx
                        .deps_data
                        .get(&function_id.module_id)
                        .unwrap();

                    dependency_data
                        .functions
                        .information
                        .iter()
                        .find(|f| &f.function_id == function_id)
                        .unwrap()
                };
                if function_information.is_native {
                    let native_function_id =
                        NativeFunction::get(&function_id.identifier, module, compilation_ctx);
                    builder.call(native_function_id);
                } else {
                    let table_id = function_table.get_table_id();
                    let f_entry =
                        function_table.add(module, function_id.clone(), function_information);
                    functions_calls_to_link.push(function_id.clone());

                    call_indirect(
                        f_entry,
                        &module_data.functions.returns[function_handle_index.into_index()],
                        table_id,
                        builder,
                        module,
                        compilation_ctx,
                    );
                }
            };

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
                local_type.box_local_instructions(module, builder, compilation_ctx, local);
            }
            types_stack.pop_expecting(local_type)?;
        }
        Bytecode::MoveLoc(local_id) => {
            // TODO: Find a way to ensure they will not be used again, the Move compiler should do the work for now
            let local = function_locals[*local_id as usize];
            let local_type = mapped_function.get_local_ir(*local_id as usize).clone();
            local_type.move_local_instructions(builder, compilation_ctx, local);
            types_stack.push(local_type);
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
            );
            types_stack.push(local_type);
        }
        Bytecode::ImmBorrowLoc(local_id) => {
            let local = function_locals[*local_id as usize];
            let local_type = mapped_function.get_local_ir(*local_id as usize).clone();
            local_type.add_borrow_local_instructions(builder, local);
            types_stack.push(IntermediateType::IRef(Box::new(local_type.clone())));
        }
        Bytecode::MutBorrowLoc(local_id) => {
            let local = function_locals[*local_id as usize];
            let local_type = mapped_function.get_local_ir(*local_id as usize).clone();
            local_type.add_borrow_local_instructions(builder, local);
            types_stack.push(IntermediateType::IMutRef(Box::new(local_type.clone())));
        }
        Bytecode::ImmBorrowField(field_id) => {
            let struct_ = module_data.structs.get_by_field_handle_idx(field_id)?;

            // Check if in the types stack we have the correct type
            types_stack.pop_expecting(&IntermediateType::IRef(Box::new(
                IntermediateType::IStruct(struct_.index()),
            )))?;

            bytecodes::structs::borrow_field(
                struct_,
                field_id,
                builder,
                compilation_ctx,
                types_stack,
            );
        }
        Bytecode::ImmBorrowFieldGeneric(field_id) => {
            let (struct_field_id, instantiation_types) = module_data
                .structs
                .instantiated_fields_to_generic_fields
                .get(field_id)
                .unwrap();

            let struct_ = if let Ok(struct_) = module_data
                .structs
                .get_struct_instance_by_field_handle_idx(field_id)
            {
                struct_
            } else {
                let generic_stuct = module_data
                    .structs
                    .get_by_field_handle_idx(struct_field_id)?;

                generic_stuct.instantiate(instantiation_types)
            };

            // Check if in the types stack we have the correct type
            types_stack.pop_expecting(&IntermediateType::IRef(Box::new(
                IntermediateType::IGenericStructInstance(
                    struct_.index(),
                    instantiation_types.to_vec(),
                ),
            )))?;

            bytecodes::structs::borrow_field(
                &struct_,
                struct_field_id,
                builder,
                compilation_ctx,
                types_stack,
            );
        }
        Bytecode::MutBorrowField(field_id) => {
            let struct_ = module_data.structs.get_by_field_handle_idx(field_id)?;

            // Check if in the types stack we have the correct type
            types_stack.pop_expecting(&IntermediateType::IMutRef(Box::new(
                IntermediateType::IStruct(struct_.index()),
            )))?;

            bytecodes::structs::mut_borrow_field(
                struct_,
                field_id,
                builder,
                compilation_ctx,
                types_stack,
            );
        }
        Bytecode::MutBorrowFieldGeneric(field_id) => {
            let (struct_field_id, instantiation_types) = module_data
                .structs
                .instantiated_fields_to_generic_fields
                .get(field_id)
                .unwrap();

            let struct_ = if let Ok(struct_) = module_data
                .structs
                .get_struct_instance_by_field_handle_idx(field_id)
            {
                struct_
            } else {
                let generic_stuct = module_data
                    .structs
                    .get_by_field_handle_idx(struct_field_id)?;
                generic_stuct.instantiate(instantiation_types)
            };

            // Check if in the types stack we have the correct type
            types_stack.pop_expecting(&IntermediateType::IMutRef(Box::new(
                IntermediateType::IGenericStructInstance(
                    struct_.index(),
                    instantiation_types.to_vec(),
                ),
            )))?;

            bytecodes::structs::mut_borrow_field(
                &struct_,
                struct_field_id,
                builder,
                compilation_ctx,
                types_stack,
            );
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

            IVector::vec_borrow_instructions(&vec_inner, module, builder, compilation_ctx);

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

            IVector::vec_borrow_instructions(&vec_inner, module, builder, compilation_ctx);

            types_stack.push(IntermediateType::IMutRef(Box::new(*vec_inner)));
        }
        Bytecode::VecPack(signature_index, num_elements) => {
            let inner =
                bytecodes::vectors::get_inner_type_from_signature(signature_index, module_data)?;

            IVector::vec_pack_instructions(
                &inner,
                module,
                builder,
                compilation_ctx,
                *num_elements as i32,
            );

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

            if *vec_inner != expected_vec_inner {
                return Err(TranslationError::TypeMismatch {
                    expected: expected_vec_inner,
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
                | IntermediateType::IExternalUserData { .. }
                | IntermediateType::IStruct(_)
                | IntermediateType::IGenericStructInstance(_, _)
                | IntermediateType::IVector(_) => {
                    let pop_back_f =
                        RuntimeFunction::VecPopBack32.get(module, Some(compilation_ctx));
                    builder.call(pop_back_f);
                }
                IntermediateType::IU64 => {
                    let pop_back_f =
                        RuntimeFunction::VecPopBack64.get(module, Some(compilation_ctx));
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
                IntermediateType::IEnum(_) => todo!(),
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

            if *vec_inner != expected_elem_type {
                return Err(TranslationError::TypeMismatch {
                    expected: expected_elem_type,
                    found: *vec_inner,
                });
            }

            if elem_ty != expected_elem_type {
                return Err(TranslationError::TypeMismatch {
                    expected: expected_elem_type,
                    found: elem_ty,
                });
            }

            IVector::vec_push_back_instructions(
                &elem_ty,
                module,
                builder,
                compilation_ctx,
                module_data,
            );
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

            if *vec_inner != expected_vec_inner {
                return Err(TranslationError::TypeMismatch {
                    expected: expected_vec_inner,
                    found: *vec_inner,
                });
            }

            match *vec_inner {
                IntermediateType::IU64 => {
                    let swap_f = RuntimeFunction::VecSwap64.get(module, Some(compilation_ctx));
                    builder.call(swap_f);
                }
                _ => {
                    let swap_f = RuntimeFunction::VecSwap32.get(module, Some(compilation_ctx));
                    builder.call(swap_f);
                }
            }
        }
        Bytecode::VecLen(signature_index) => {
            let elem_ir_type =
                bytecodes::vectors::get_inner_type_from_signature(signature_index, module_data)?;

            types_stack.pop_expecting(&IntermediateType::IRef(Box::new(
                IntermediateType::IVector(Box::new(elem_ir_type)),
            )))?;

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

            inner.add_read_ref_instructions(builder, module, compilation_ctx, module_data);
            types_stack.push(*inner);
        }
        Bytecode::WriteRef => {
            let [iref, value_type] = types_stack.pop_n_from_stack()?;

            types_stack::match_types!((IntermediateType::IMutRef(inner), "IMutRef", iref));

            if *inner == value_type {
                inner.add_write_ref_instructions(module, builder, compilation_ctx);
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
            prepare_function_return(
                module,
                builder,
                &mapped_function.signature.returns,
                compilation_ctx,
            );

            // We dont pop the return values from the stack, we just check if the types match
            assert!(
                types_stack.0.ends_with(&mapped_function.signature.returns),
                "types stack does not match function return types"
            );
        }
        Bytecode::CastU8 => {
            let original_type = types_stack.pop()?;
            IU8::cast_from(builder, module, original_type, compilation_ctx);
            types_stack.push(IntermediateType::IU8);
        }
        Bytecode::CastU16 => {
            let original_type = types_stack.pop()?;
            IU16::cast_from(builder, module, original_type, compilation_ctx);
            types_stack.push(IntermediateType::IU16);
        }
        Bytecode::CastU32 => {
            let original_type = types_stack.pop()?;
            IU32::cast_from(builder, module, original_type, compilation_ctx);
            types_stack.push(IntermediateType::IU32);
        }
        Bytecode::CastU64 => {
            let original_type = types_stack.pop()?;
            IU64::cast_from(builder, module, original_type, compilation_ctx);
            types_stack.push(IntermediateType::IU64);
        }
        Bytecode::CastU128 => {
            let original_type = types_stack.pop()?;
            IU128::cast_from(builder, module, original_type, compilation_ctx);
            types_stack.push(IntermediateType::IU128);
        }
        Bytecode::CastU256 => {
            let original_type = types_stack.pop()?;
            IU256::cast_from(builder, module, original_type, compilation_ctx);
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
                IntermediateType::IU8 => IU8::add(builder, module),
                IntermediateType::IU16 => IU16::add(builder, module),
                IntermediateType::IU32 => IU32::add(builder, module),
                IntermediateType::IU64 => IU64::add(builder, module),
                IntermediateType::IU128 => IU128::add(builder, module, compilation_ctx),
                IntermediateType::IU256 => IU256::add(builder, module, compilation_ctx),
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
                IntermediateType::IU8 => IU8::sub(builder, module),
                IntermediateType::IU16 => IU16::sub(builder, module),
                IntermediateType::IU32 => IU32::sub(builder, module),
                IntermediateType::IU64 => IU64::sub(builder, module),
                IntermediateType::IU128 => IU128::sub(builder, module, compilation_ctx),
                IntermediateType::IU256 => IU256::sub(builder, module, compilation_ctx),
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
                IntermediateType::IU8 => IU8::mul(builder, module),
                IntermediateType::IU16 => IU16::mul(builder, module),
                IntermediateType::IU32 => IU32::mul(builder, module),
                IntermediateType::IU64 => IU64::mul(builder, module),
                IntermediateType::IU128 => IU128::mul(builder, module, compilation_ctx),
                IntermediateType::IU256 => IU256::mul(builder, module, compilation_ctx),
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
                IntermediateType::IU128 => IU128::div(builder, module, compilation_ctx),
                IntermediateType::IU256 => IU256::div(builder, module, compilation_ctx),
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
                    let less_than_f = RuntimeFunction::LessThan.get(module, Some(compilation_ctx));

                    builder.i32_const(IU128::HEAP_SIZE).call(less_than_f);
                }
                IntermediateType::IU256 => {
                    let less_than_f = RuntimeFunction::LessThan.get(module, Some(compilation_ctx));

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
                    let less_than_f = RuntimeFunction::LessThan.get(module, Some(compilation_ctx));

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
                    let less_than_f = RuntimeFunction::LessThan.get(module, Some(compilation_ctx));

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
                    let less_than_f = RuntimeFunction::LessThan.get(module, Some(compilation_ctx));

                    let a = module.locals.add(ValType::I32);
                    let b = module.locals.add(ValType::I32);

                    builder
                        .swap(a, b)
                        .i32_const(IU128::HEAP_SIZE)
                        .call(less_than_f);
                }
                IntermediateType::IU256 => {
                    let less_than_f = RuntimeFunction::LessThan.get(module, Some(compilation_ctx));

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
                    let less_than_f = RuntimeFunction::LessThan.get(module, Some(compilation_ctx));

                    // Compare
                    builder
                        .i32_const(IU128::HEAP_SIZE)
                        .call(less_than_f)
                        .negate();
                }
                IntermediateType::IU256 => {
                    let less_than_f = RuntimeFunction::LessThan.get(module, Some(compilation_ctx));

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
                IntermediateType::IU128 => IU128::remainder(builder, module, compilation_ctx),
                IntermediateType::IU256 => IU256::remainder(builder, module, compilation_ctx),
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

            t1.load_equality_instructions(module, builder, compilation_ctx, module_data);

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

            t1.load_not_equality_instructions(module, builder, compilation_ctx, module_data);

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
                IntermediateType::IU8 => IU8::bit_shift_left(builder, module),
                IntermediateType::IU16 => IU16::bit_shift_left(builder, module),
                IntermediateType::IU32 => IU32::bit_shift_left(builder, module),
                IntermediateType::IU64 => IU64::bit_shift_left(builder, module),
                IntermediateType::IU128 => IU128::bit_shift_left(builder, module, compilation_ctx),
                IntermediateType::IU256 => IU256::bit_shift_left(builder, module, compilation_ctx),
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
                IntermediateType::IU8 => IU8::bit_shift_right(builder, module),
                IntermediateType::IU16 => IU16::bit_shift_right(builder, module),
                IntermediateType::IU32 => IU32::bit_shift_right(builder, module),
                IntermediateType::IU64 => IU64::bit_shift_right(builder, module),
                IntermediateType::IU128 => IU128::bit_shift_right(builder, module, compilation_ctx),
                IntermediateType::IU256 => IU256::bit_shift_right(builder, module, compilation_ctx),
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

            bytecodes::structs::pack(struct_, module, builder, compilation_ctx, types_stack)?;

            types_stack.push(IntermediateType::IStruct(struct_definition_index.0));
        }
        Bytecode::PackGeneric(struct_definition_index) => {
            let struct_ = module_data
                .structs
                .get_struct_instance_by_struct_definition_idx(struct_definition_index)?;

            bytecodes::structs::pack(&struct_, module, builder, compilation_ctx, types_stack)?;

            let idx = module_data
                .structs
                .get_generic_struct_idx_by_struct_definition_idx(struct_definition_index);
            let types = module_data
                .structs
                .get_generic_struct_types_instances(struct_definition_index)?;

            types_stack.push(IntermediateType::IGenericStructInstance(
                idx,
                types.to_vec(),
            ));
        }
        Bytecode::Unpack(struct_definition_index) => {
            types_stack.pop_expecting(&IntermediateType::IStruct(struct_definition_index.0))?;

            let struct_ = module_data
                .structs
                .get_by_struct_definition_idx(struct_definition_index)?;

            bytecodes::structs::unpack(struct_, module, builder, compilation_ctx, types_stack)?;
        }
        Bytecode::UnpackGeneric(struct_definition_index) => {
            let idx = module_data
                .structs
                .get_generic_struct_idx_by_struct_definition_idx(struct_definition_index);
            let types = module_data
                .structs
                .get_generic_struct_types_instances(struct_definition_index)?;

            types_stack.pop_expecting(&IntermediateType::IGenericStructInstance(
                idx,
                types.to_vec(),
            ))?;

            let struct_ = module_data
                .structs
                .get_struct_instance_by_struct_definition_idx(struct_definition_index)?;

            bytecodes::structs::unpack(&struct_, module, builder, compilation_ctx, types_stack)?;
        }
        Bytecode::BrTrue(code_offset) => {
            if let Some(branch_mode) = branches.get(code_offset) {
                if let Some(&target) = branch_targets.get_target(branch_mode, code_offset) {
                    builder.br_if(target);
                }
            }
        }
        Bytecode::BrFalse(code_offset) => {
            if let Some(branch_mode) = branches.get(code_offset) {
                if let Some(&target) = branch_targets.get_target(branch_mode, code_offset) {
                    builder.unop(UnaryOp::I32Eqz);
                    builder.br_if(target);
                }
            }
        }
        Bytecode::Branch(code_offset) => {
            if let Some(branch_mode) = branches.get(code_offset) {
                if let Some(&target) = branch_targets.get_target(branch_mode, code_offset) {
                    builder.br(target);
                }
            }
        }
        //**
        // Enums
        //**
        Bytecode::PackVariant(index) => {
            let enum_ = module_data.enums.get_enum_by_variant_handle_idx(index)?;
            let index_inside_enum = module_data
                .enums
                .get_variant_position_by_variant_handle_idx(index)?;

            bytecodes::enums::pack_variant(
                enum_,
                index_inside_enum,
                module,
                builder,
                compilation_ctx,
                types_stack,
            )?;

            types_stack.push(IntermediateType::IEnum(enum_.index));
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
) {
    builder
        .i32_const(function_entry.index)
        .call_indirect(function_entry.type_id, wasm_table_id);

    add_unpack_function_return_values_instructions(
        builder,
        module,
        function_returns,
        compilation_ctx.memory_id,
    );
}

fn process_fn_local_variables(
    function_information: &MappedFunction,
    module: &mut Module,
) -> (Vec<LocalId>, Vec<LocalId>) {
    let wasm_arg_types = function_information.signature.get_argument_wasm_types();
    let wasm_ret_types = function_information.signature.get_return_wasm_types();

    assert!(
        wasm_ret_types.len() <= 1,
        "Multiple return values not supported"
    );

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
                IntermediateType::IU64 => ValType::I32, // to store pointer instead of i64
                other => ValType::from(other),
            }
        })
        .map(|ty| module.locals.add(ty))
        .collect();

    (wasm_arg_locals, wasm_declared_locals)
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
) {
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
                ty.box_local_instructions(module, builder, compilation_ctx, outer_ptr);

                if let Some(index) = function_locals.iter().position(|&id| id == *local) {
                    updates.push((index, outer_ptr));
                } else {
                    panic!(
                        "Couldn't find original local {:?} in function_information",
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
        function_locals[index] = pointer;
    }
}
