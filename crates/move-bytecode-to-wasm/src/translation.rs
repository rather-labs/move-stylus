use std::collections::HashMap;

use anyhow::Result;
use move_binary_format::file_format::Bytecode;
use relooper::BranchMode;
use walrus::ir::{BinaryOp, Block, InstrSeqId, InstrSeqType, LoadKind, MemArg, UnaryOp};
use walrus::{FunctionBuilder, FunctionId, InstrSeqBuilder, Module, ValType};

use crate::CompilationContext;
use crate::runtime::RuntimeFunction;
use crate::wasm_builder_extensions::WasmBuilderExtension;

use flow::Flow;
use functions::{
    MappedFunction, add_unpack_function_return_values_instructions, prepare_function_return,
};
use intermediate_types::IntermediateType;
use intermediate_types::heap_integers::{IU128, IU256};
use intermediate_types::simple_integers::{IU8, IU16, IU32, IU64};
use intermediate_types::vector::IVector;
use table::{FunctionTable, TableEntry};
use types_stack::TypesStack;

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

pub fn translate_function(
    module: &mut Module,
    index: usize,
    compilation_ctx: &CompilationContext,
    function_table: &mut FunctionTable,
) -> Result<FunctionId> {
    let entry = function_table
        .get_mut(index)
        .ok_or(anyhow::anyhow!("index {index} not found in function table"))?;

    anyhow::ensure!(
        entry.get_move_code_unit().unwrap().jump_tables.is_empty(),
        "Jump tables are not supported yet"
    );

    let mut function = FunctionBuilder::new(&mut module.types, &entry.params, &entry.results);
    let mut builder = function.func_body();

    entry
        .function
        .box_args(&mut builder, module, compilation_ctx);

    let entry = function_table
        .get(index)
        .ok_or(anyhow::anyhow!("index {index} not found in function table"))?;

    let code_unit = &entry.get_move_code_unit().unwrap();

    let flow = Flow::new(code_unit, compilation_ctx, &entry.function);
    println!("{flow:#?}");

    // Type stack for the current function.
    // It is filled recursively, meaning parent nodes inherit types left by child nodes on the stack.
    let mut types_stack = TypesStack::new();
    // Loop targets maps the relooper-assigned loop id to the loop's instruction sequence id.
    let mut loop_targets: HashMap<u16, InstrSeqId> = HashMap::new();

    translate_flow(
        compilation_ctx,
        &mut builder,
        module,
        function_table,
        &mut types_stack,
        &flow,
        &mut loop_targets,
        entry,
    );

    let function_id = function.finish(entry.function.arg_locals.clone(), &mut module.funcs);
    Ok(function_id)
}

// TODO: check we are setting result types correctly
fn translate_flow(
    compilation_ctx: &CompilationContext,
    builder: &mut InstrSeqBuilder,
    module: &mut Module,
    function_table: &FunctionTable,
    types_stack: &mut TypesStack,
    flow: &Flow,
    loop_targets: &mut HashMap<u16, InstrSeqId>,
    entry: &TableEntry,
) {
    match flow {
        Flow::Simple {
            instructions,
            branches,
            ..
        } => {
            for instruction in instructions {
                match instruction {
                    // WATCH OUT, BRFALSE CAN ALSO BE USED AS A LOOP CONTINUE
                    Bytecode::Branch(code_offset) | Bytecode::BrFalse(code_offset) => {
                        if let Some(branch_mode) = branches.get(code_offset) {
                            // TODO: WTF are multi branch modes?
                            let loop_id = match branch_mode {
                                BranchMode::LoopContinue(id)
                                | BranchMode::LoopContinueIntoMulti(id) => Some(id),
                                _ => None,
                            };

                            if let Some(loop_id) = loop_id {
                                if let Some(target_block_id) = loop_targets.get(loop_id) {
                                    match instruction {
                                        Bytecode::Branch(code_offset) => {
                                            builder.br(*target_block_id);
                                        }
                                        Bytecode::BrFalse(code_offset) => {
                                            builder.unop(UnaryOp::I32Eqz); // i32.eqz
                                            builder.br_if(*target_block_id);
                                        }
                                        _ => {}
                                    }
                                } else {
                                    panic!("Loop target not found for loop_id: {}", loop_id);
                                }
                            }
                        }
                    }

                    _ => {
                        translate_instruction(
                            instruction,
                            compilation_ctx,
                            builder,
                            &entry.function,
                            module,
                            function_table,
                            types_stack,
                            loop_targets,
                        )
                        .unwrap();
                    }
                }
            }
        }
        Flow::Sequence(flows) => {
            for f in flows {
                translate_flow(
                    compilation_ctx,
                    builder,
                    module,
                    function_table,
                    types_stack,
                    f,
                    loop_targets,
                    entry,
                );
            }
        }
        Flow::Loop {
            types_stack,
            loop_id,
            body,
        } => {
            let ty = InstrSeqType::new(&mut module.types, &[], &types_stack.to_val_types());

            // TODO: Consider enclosing the loop within a block to facilitate mapping LoopBreak branch modes for exiting the loop.
            // builder.block(ty, |block| {
            builder.loop_(ty, |loop_| {
                // loop_targets maps the relooper-assigned loop id to the loop's instruction sequence id.
                loop_targets.insert(*loop_id, loop_.id());

                translate_flow(
                    compilation_ctx,
                    loop_,
                    module,
                    function_table,
                    &mut types_stack.clone(),
                    &*body,
                    loop_targets,
                    entry,
                );
            });
            // });
        }
        // TODO: currently we are wrapping the instructions within each [if, else] branch in a block, which is not desired but required by walrus.
        // If possible, it would be great to avoid this.
        Flow::IfElse {
            types_stack,
            then_body,
            else_body,
        } => {
            let then_types_stack = then_body.get_types_stack();
            let else_types_stack = else_body.get_types_stack();
            let then_val_types =
                InstrSeqType::new(&mut module.types, &[], &then_types_stack.to_val_types());
            let else_val_types =
                InstrSeqType::new(&mut module.types, &[], &else_types_stack.to_val_types());

            let then_id = {
                let mut then_seq = builder.dangling_instr_seq(then_val_types);
                translate_flow(
                    compilation_ctx,
                    &mut then_seq,
                    module,
                    function_table,
                    &mut types_stack.clone(),
                    &*then_body,
                    loop_targets,
                    entry,
                );
                then_seq.id()
            };

            let else_id = {
                let mut else_seq = builder.dangling_instr_seq(else_val_types);
                translate_flow(
                    compilation_ctx,
                    &mut else_seq,
                    module,
                    function_table,
                    &mut types_stack.clone(),
                    &*else_body,
                    loop_targets,
                    entry,
                );
                else_seq.id()
            };

            if then_types_stack == else_types_stack {
                builder.if_else(
                    then_val_types,
                    |then| {
                        then.instr(Block { seq: then_id });
                    },
                    |else_| {
                        else_.instr(Block { seq: else_id });
                    },
                );
            } else if then_types_stack.is_empty() {
                builder.if_else(
                    None,
                    |then| {
                        then.instr(Block { seq: then_id });
                    },
                    |else_| {},
                );
                builder.instr(Block { seq: else_id });
            } else if else_types_stack.is_empty() {
                builder.if_else(
                    None,
                    |then| {},
                    |else_| {
                        else_.instr(Block { seq: else_id });
                    },
                );
                builder.instr(Block { seq: then_id });
            } else {
                panic!(
                    "Error: Mismatched types on the stack from Then and Else branches, and neither is empty."
                );
            }
        }
        Flow::Empty => (),
    }
}

fn translate_instruction(
    instruction: &Bytecode,
    compilation_ctx: &CompilationContext,
    builder: &mut InstrSeqBuilder,
    mapped_function: &MappedFunction,
    module: &mut Module,
    function_table: &FunctionTable,
    types_stack: &mut TypesStack,
    loop_targets: &HashMap<u16, InstrSeqId>,
) -> Result<(), TranslationError> {
    match instruction {
        // Load a fixed constant
        Bytecode::LdConst(global_index) => {
            let constant = &compilation_ctx.constants[global_index.0 as usize];
            let mut data = constant.data.clone().into_iter();
            let constant_type = &constant.type_;
            let constant_type: IntermediateType = IntermediateType::try_from_signature_token(
                constant_type,
                compilation_ctx.datatype_handles_map,
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
            let arguments = &compilation_ctx.functions_arguments[function_handle_index.0 as usize];
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

            let f = function_table
                .get_by_function_handle_index(function_handle_index)
                .expect("function with index {function_handle_index:?} not found un table");

            builder
                .i32_const(f.index)
                .call_indirect(f.type_id, function_table.get_table_id());

            add_unpack_function_return_values_instructions(
                builder,
                module,
                &compilation_ctx.functions_returns[function_handle_index.0 as usize],
                compilation_ctx.memory_id,
            );
            // Insert in the stack types the types returned by the function (if any)
            let return_types = &compilation_ctx.functions_returns[function_handle_index.0 as usize];
            types_stack.append(return_types);
        }
        // Locals
        Bytecode::StLoc(local_id) => {
            let local = mapped_function.function_locals[*local_id as usize];
            let local_type = &mapped_function.function_locals_ir[*local_id as usize];
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
            let local = mapped_function.function_locals[*local_id as usize];
            let local_type = mapped_function.function_locals_ir[*local_id as usize].clone();
            local_type.move_local_instructions(builder, compilation_ctx, local);
            types_stack.push(local_type);
        }
        Bytecode::CopyLoc(local_id) => {
            let local = mapped_function.function_locals[*local_id as usize];
            let local_type = mapped_function.function_locals_ir[*local_id as usize].clone();
            local_type.copy_local_instructions(module, builder, compilation_ctx, local);
            types_stack.push(local_type);
        }
        Bytecode::ImmBorrowLoc(local_id) => {
            let local = mapped_function.function_locals[*local_id as usize];
            let local_type = &mapped_function.function_locals_ir[*local_id as usize];
            local_type.add_borrow_local_instructions(builder, local);
            types_stack.push(IntermediateType::IRef(Box::new(local_type.clone())));
        }
        Bytecode::MutBorrowLoc(local_id) => {
            let local = mapped_function.function_locals[*local_id as usize];
            let local_type = &mapped_function.function_locals_ir[*local_id as usize];
            local_type.add_borrow_local_instructions(builder, local);
            types_stack.push(IntermediateType::IMutRef(Box::new(local_type.clone())));
        }
        Bytecode::ImmBorrowField(field_id) => {
            let struct_ = compilation_ctx.get_struct_by_field_handle_idx(field_id)?;

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
            let (struct_field_id, instantiation_types) = compilation_ctx
                .instantiated_fields_to_generic_fields
                .get(field_id)
                .unwrap();

            let instantiation_types = instantiation_types
                .iter()
                .map(|t| {
                    IntermediateType::try_from_signature_token(
                        t,
                        compilation_ctx.datatype_handles_map,
                    )
                })
                .collect::<Result<Vec<_>, anyhow::Error>>()?;

            let struct_ = if let Ok(struct_) =
                compilation_ctx.get_generic_struct_by_field_handle_idx(field_id)
            {
                struct_
            } else {
                let generic_stuct =
                    compilation_ctx.get_struct_by_field_handle_idx(struct_field_id)?;

                generic_stuct.instantiate(&instantiation_types)
            };

            // Check if in the types stack we have the correct type
            types_stack.pop_expecting(&IntermediateType::IRef(Box::new(
                IntermediateType::IGenericStructInstance(struct_.index(), instantiation_types),
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
            let struct_ = compilation_ctx.get_struct_by_field_handle_idx(field_id)?;

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
            let (struct_field_id, instantiation_types) = compilation_ctx
                .instantiated_fields_to_generic_fields
                .get(field_id)
                .unwrap();

            let instantiation_types = instantiation_types
                .iter()
                .map(|t| {
                    IntermediateType::try_from_signature_token(
                        t,
                        compilation_ctx.datatype_handles_map,
                    )
                })
                .collect::<Result<Vec<_>, anyhow::Error>>()?;

            let struct_ = if let Ok(struct_) =
                compilation_ctx.get_generic_struct_by_field_handle_idx(field_id)
            {
                struct_
            } else {
                let generic_stuct =
                    compilation_ctx.get_struct_by_field_handle_idx(struct_field_id)?;
                generic_stuct.instantiate(&instantiation_types)
            };

            // Check if in the types stack we have the correct type
            types_stack.pop_expecting(&IntermediateType::IMutRef(Box::new(
                IntermediateType::IGenericStructInstance(struct_.index(), instantiation_types),
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

            let expected_vec_inner = bytecodes::vectors::get_inner_type_from_signature(
                signature_index,
                compilation_ctx,
            )?;

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

            let expected_vec_inner = bytecodes::vectors::get_inner_type_from_signature(
                signature_index,
                compilation_ctx,
            )?;

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
            let inner = bytecodes::vectors::get_inner_type_from_signature(
                signature_index,
                compilation_ctx,
            )?;

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

            let expected_vec_inner = bytecodes::vectors::get_inner_type_from_signature(
                signature_index,
                compilation_ctx,
            )?;

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

            let expected_elem_type = bytecodes::vectors::get_inner_type_from_signature(
                signature_index,
                compilation_ctx,
            )?;

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

            IVector::vec_push_back_instructions(&elem_ty, module, builder, compilation_ctx);
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

            let expected_vec_inner = bytecodes::vectors::get_inner_type_from_signature(
                signature_index,
                compilation_ctx,
            )?;

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
            let elem_ir_type = bytecodes::vectors::get_inner_type_from_signature(
                signature_index,
                compilation_ctx,
            )?;

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

            inner.add_read_ref_instructions(builder, module, compilation_ctx);
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

            t1.load_equality_instructions(module, builder, compilation_ctx);

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

            t1.load_not_equality_instructions(module, builder, compilation_ctx);

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
            let struct_ =
                compilation_ctx.get_struct_by_struct_definition_idx(struct_definition_index)?;

            bytecodes::structs::pack(struct_, module, builder, compilation_ctx, types_stack)?;

            types_stack.push(IntermediateType::IStruct(struct_definition_index.0));
        }
        Bytecode::PackGeneric(struct_definition_index) => {
            let struct_ = compilation_ctx
                .get_generic_struct_by_struct_definition_idx(struct_definition_index)?;

            bytecodes::structs::pack(&struct_, module, builder, compilation_ctx, types_stack)?;

            let idx = compilation_ctx
                .get_generic_struct_idx_by_struct_definition_idx(struct_definition_index);
            let types =
                compilation_ctx.get_generic_struct_types_instances(struct_definition_index)?;

            types_stack.push(IntermediateType::IGenericStructInstance(idx, types));
        }
        Bytecode::Unpack(struct_definition_index) => {
            types_stack.pop_expecting(&IntermediateType::IStruct(struct_definition_index.0))?;

            let struct_ =
                compilation_ctx.get_struct_by_struct_definition_idx(struct_definition_index)?;

            bytecodes::structs::unpack(struct_, module, builder, compilation_ctx, types_stack)?;
        }
        Bytecode::UnpackGeneric(struct_definition_index) => {
            let idx = compilation_ctx
                .get_generic_struct_idx_by_struct_definition_idx(struct_definition_index);
            let types =
                compilation_ctx.get_generic_struct_types_instances(struct_definition_index)?;

            types_stack.pop_expecting(&IntermediateType::IGenericStructInstance(idx, types))?;

            let struct_ = compilation_ctx
                .get_generic_struct_by_struct_definition_idx(struct_definition_index)?;

            bytecodes::structs::unpack(&struct_, module, builder, compilation_ctx, types_stack)?;
        }
        Bytecode::BrTrue(_) | Bytecode::BrFalse(_) | Bytecode::Branch(_) => {}
        b => Err(TranslationError::UnsupportedOperation {
            operation: b.clone(),
        })?,
    }

    Ok(())
}
