use anyhow::Result;
use functions::{
    MappedFunction, add_unpack_function_return_values_instructions, prepare_function_return,
};
use intermediate_types::IntermediateType;
use intermediate_types::heap_integers::{IU128, IU256};
use intermediate_types::simple_integers::{IU16, IU32, IU64};
use intermediate_types::{simple_integers::IU8, vector::IVector};
use move_binary_format::file_format::{Bytecode, SignatureIndex};
use table::FunctionTable;
use types_stack::TypesStack;
use walrus::ir::{BinaryOp, LoadKind, UnaryOp};
use walrus::{FunctionBuilder, Module};
use walrus::{FunctionId, InstrSeqBuilder, ValType, ir::MemArg};

use crate::CompilationContext;
use crate::runtime::RuntimeFunction;
use crate::wasm_builder_extensions::WasmBuilderExtension;

pub(crate) mod bytecodes;
pub mod error;
pub mod functions;
/// The types in this module represent an intermediate Rust representation of Move types
/// that is used to generate the WASM code.
pub mod intermediate_types;
pub mod table;
pub(crate) mod types_stack;
pub use error::TranslationError;

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

    let code = &entry.get_move_code_unit().unwrap().code;

    let mut types_stack = TypesStack::new();

    for instruction in code {
        map_bytecode_instruction(
            instruction,
            compilation_ctx,
            &mut builder,
            &entry.function,
            module,
            function_table,
            &mut types_stack,
        )
        .expect("There was an error translating the bytecode instruction {instruction:?}");
    }

    let function_id = function.finish(entry.function.arg_locals.clone(), &mut module.funcs);
    Ok(function_id)
}

fn map_bytecode_instruction(
    instruction: &Bytecode,
    compilation_ctx: &CompilationContext,
    builder: &mut InstrSeqBuilder,
    mapped_function: &MappedFunction,
    module: &mut Module,
    function_table: &FunctionTable,
    types_stack: &mut TypesStack,
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
            )
            .unwrap();

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
            for (i, argument) in arguments.iter().enumerate().rev() {
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
            let struct_ = compilation_ctx
                .get_struct_by_field_handle_idx(field_id)
                .unwrap();

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

            let struct_ = if let Ok(struct_) =
                compilation_ctx.get_generic_struct_by_field_handle_idx(field_id)
            {
                struct_
            } else {
                let generic_stuct = compilation_ctx
                    .get_struct_by_field_handle_idx(struct_field_id)
                    .unwrap();
                let instantiation_types = instantiation_types
                    .iter()
                    .map(|t| {
                        IntermediateType::try_from_signature_token(
                            t,
                            compilation_ctx.datatype_handles_map,
                        )
                    })
                    .collect::<Result<Vec<_>, anyhow::Error>>()
                    .unwrap();
                generic_stuct.instantiate(&instantiation_types)
            };

            // Check if in the types stack we have the correct type
            types_stack.pop_expecting(&IntermediateType::IRef(Box::new(
                IntermediateType::IGenericStructInstance(struct_.index(), struct_.fields),
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
            let struct_ = compilation_ctx
                .get_struct_by_field_handle_idx(field_id)
                .unwrap();

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

            let struct_ = if let Ok(struct_) =
                compilation_ctx.get_generic_struct_by_field_handle_idx(field_id)
            {
                struct_
            } else {
                let generic_stuct = compilation_ctx
                    .get_struct_by_field_handle_idx(struct_field_id)
                    .unwrap();
                let instantiation_types = instantiation_types
                    .iter()
                    .map(|t| {
                        IntermediateType::try_from_signature_token(
                            t,
                            compilation_ctx.datatype_handles_map,
                        )
                    })
                    .collect::<Result<Vec<_>, anyhow::Error>>()
                    .unwrap();
                generic_stuct.instantiate(&instantiation_types)
            };

            // Check if in the types stack we have the correct type
            types_stack.pop_expecting(&IntermediateType::IMutRef(Box::new(
                IntermediateType::IGenericStructInstance(struct_.index(), struct_.fields),
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

            types_stack::match_n_types!(
                (IntermediateType::IU64, "u64", t1),
                (IntermediateType::IRef(ref_inner), "vector reference", t2),
                (IntermediateType::IVector(vec_inner), "vector", *ref_inner)
            );

            let expected_vec_inner = get_ir_for_signature_index(compilation_ctx, *signature_index);

            if *vec_inner != expected_vec_inner {
                panic!(
                    "Expected vector inner type {:?}, got {:?}",
                    expected_vec_inner, *vec_inner
                );
            }

            IVector::vec_borrow_instructions(&vec_inner, module, builder, compilation_ctx);

            types_stack.push(IntermediateType::IRef(Box::new(*vec_inner)));
        }
        Bytecode::VecMutBorrow(signature_index) => {
            let [t1, t2] = types_stack.pop_n_from_stack()?;

            types_stack::match_n_types!(
                (IntermediateType::IU64, "u64", t1),
                (
                    IntermediateType::IMutRef(ref_inner),
                    "mutable vector reference",
                    t2
                ),
                (IntermediateType::IVector(vec_inner), "vector", *ref_inner)
            );

            let expected_vec_inner = get_ir_for_signature_index(compilation_ctx, *signature_index);

            if *vec_inner != expected_vec_inner {
                panic!(
                    "Expected vector inner type {:?}, got {:?}",
                    expected_vec_inner, *vec_inner
                );
            }

            IVector::vec_borrow_instructions(&vec_inner, module, builder, compilation_ctx);

            types_stack.push(IntermediateType::IMutRef(Box::new(*vec_inner)));
        }
        Bytecode::VecPack(signature_index, num_elements) => {
            let inner = get_ir_for_signature_index(compilation_ctx, *signature_index);
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

            types_stack::match_n_types!(
                (
                    IntermediateType::IMutRef(ref_inner),
                    "mutable vector reference",
                    ty
                ),
                (IntermediateType::IVector(vec_inner), "vector", *ref_inner)
            );

            let expected_vec_inner = get_ir_for_signature_index(compilation_ctx, *signature_index);

            if *vec_inner != expected_vec_inner {
                panic!(
                    "Expected vector inner type {:?}, got {:?}",
                    expected_vec_inner, *vec_inner
                );
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
                IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                    panic!("VecPopBack operation is not allowed on reference types");
                }
                IntermediateType::ITypeParameter(_) => {
                    panic!("can't perform VecPopBack on type parameters");
                }
            }

            types_stack.push(*vec_inner);
        }
        Bytecode::VecPushBack(signature_index) => {
            let [elem_ty, ref_ty] = types_stack.pop_n_from_stack()?;

            types_stack::match_n_types!(
                (
                    IntermediateType::IMutRef(mut_inner),
                    "mutable vector reference",
                    ref_ty
                ),
                (IntermediateType::IVector(vec_inner), "vector", *mut_inner)
            );

            let expected_elem_type = get_ir_for_signature_index(compilation_ctx, *signature_index);

            if *vec_inner != expected_elem_type {
                panic!(
                    "Expected vector inner type {:?}, got {:?}",
                    expected_elem_type, *vec_inner
                );
            }

            if elem_ty != expected_elem_type {
                panic!(
                    "Expected element type {:?}, got {:?}",
                    expected_elem_type, elem_ty
                );
            }

            IVector::vec_push_back_instructions(&elem_ty, module, builder, compilation_ctx);
        }
        Bytecode::VecSwap(signature_index) => {
            let [id2_ty, id1_ty, ref_ty] = types_stack.pop_n_from_stack()?;

            types_stack::match_n_types!(
                (IntermediateType::IU64, "u64", id2_ty),
                (IntermediateType::IU64, "u64", id1_ty),
                (
                    IntermediateType::IMutRef(mut_inner),
                    "mutable vector reference",
                    ref_ty
                ),
                (IntermediateType::IVector(vec_inner), "vector", *mut_inner)
            );

            let expected_vec_inner = get_ir_for_signature_index(compilation_ctx, *signature_index);

            if *vec_inner != expected_vec_inner {
                panic!(
                    "Expected vector inner type {:?}, got {:?}",
                    expected_vec_inner, *vec_inner
                );
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
            let elem_ir_type = get_ir_for_signature_index(compilation_ctx, *signature_index);

            let ir_type = IntermediateType::IVector(Box::new(elem_ir_type.clone()));

            let ty = types_stack.pop()?;
            types_stack::match_n_types!((
                IntermediateType::IRef(actual_type),
                "vector reference",
                ty
            ));

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

            types_stack::match_n_types!((
                (IntermediateType::IRef(inner) | IntermediateType::IMutRef(inner)),
                "IRef or IMutRef",
                ref_type
            ));

            inner.add_read_ref_instructions(builder, module, compilation_ctx);
            types_stack.push(*inner);
        }
        Bytecode::WriteRef => match (types_stack.pop(), types_stack.pop()) {
            (Some(IntermediateType::IMutRef(inner)), Some(value_type)) => {
                if *inner == value_type {
                    inner.add_write_ref_instructions(module, builder, compilation_ctx);
                } else {
                    panic!(
                        "WriteRef type mismatch: expected value of type {:?}, got {:?}",
                        inner, value_type
                    );
                }
            }
            (Some(other), Some(_)) => {
                panic!("WriteRef expected a mutable reference, got {:?}", other);
            }
            _ => panic!("Type stack underflow on WriteRef"),
        },
        Bytecode::FreezeRef => {
            let ref_type = types_stack
                .pop()
                .expect("FreezeRef expects a reference on the stack");

            match ref_type {
                IntermediateType::IMutRef(inner) => {
                    types_stack.push(IntermediateType::IRef(inner.clone()));
                }
                _ => panic!("FreezeRef expected a mutable reference, got {:?}", ref_type),
            }
        }
        Bytecode::Pop => {
            builder.drop();
            types_stack
                .pop()
                .unwrap_or_else(|| panic!("error dropping from types stack: types stack is empty"));
        }
        // TODO: ensure this is the last instruction in the move code
        Bytecode::Ret => {
            prepare_function_return(
                module,
                builder,
                &mapped_function.signature.returns,
                compilation_ctx,
            );

            // Remove the return values from types stack and check if the types are correct
            for (i, return_type) in mapped_function.signature.returns.iter().rev().enumerate() {
                if let Err(e) = pop_types_stack(types_stack, return_type) {
                    panic!("Function return type mismatch at index {i}: {e}");
                }
            }

            // Stack types should be empty
            assert!(
                types_stack.is_empty(),
                "types stack is not empty after return"
            );
        }
        Bytecode::CastU8 => {
            let original_type = types_stack.pop().unwrap();
            IU8::cast_from(builder, module, original_type, compilation_ctx);
            types_stack.push(IntermediateType::IU8);
        }
        Bytecode::CastU16 => {
            let original_type = types_stack.pop().unwrap();
            IU16::cast_from(builder, module, original_type, compilation_ctx);
            types_stack.push(IntermediateType::IU16);
        }
        Bytecode::CastU32 => {
            let original_type = types_stack.pop().unwrap();
            IU32::cast_from(builder, module, original_type, compilation_ctx);
            types_stack.push(IntermediateType::IU32);
        }
        Bytecode::CastU64 => {
            let original_type = types_stack.pop().unwrap();
            IU64::cast_from(builder, module, original_type, compilation_ctx);
            types_stack.push(IntermediateType::IU64);
        }
        Bytecode::CastU128 => {
            let original_type = types_stack.pop().unwrap();
            IU128::cast_from(builder, module, original_type, compilation_ctx);
            types_stack.push(IntermediateType::IU128);
        }
        Bytecode::CastU256 => {
            let original_type = types_stack.pop().unwrap();
            IU256::cast_from(builder, module, original_type, compilation_ctx);
            types_stack.push(IntermediateType::IU256);
        }
        Bytecode::Add => {
            let sum_type = if let (Some(t1), Some(t2)) = (types_stack.pop(), types_stack.pop()) {
                assert_eq!(
                    t1, t2,
                    "types stack error: trying two add two different types {t1:?} {t2:?}"
                );
                t1
            } else {
                panic!("types stack is empty");
            };

            match sum_type {
                IntermediateType::IU8 => IU8::add(builder, module),
                IntermediateType::IU16 => IU16::add(builder, module),
                IntermediateType::IU32 => IU32::add(builder, module),
                IntermediateType::IU64 => IU64::add(builder, module),
                IntermediateType::IU128 => IU128::add(builder, module, compilation_ctx),
                IntermediateType::IU256 => IU256::add(builder, module, compilation_ctx),
                t => panic!("type stack error: trying to add two {t:?}"),
            }

            types_stack.push(sum_type);
        }
        Bytecode::Sub => {
            let sub_type = if let (Some(t1), Some(t2)) = (types_stack.pop(), types_stack.pop()) {
                assert_eq!(
                    t1, t2,
                    "types stack error: trying two substract two different types {t1:?} {t2:?}"
                );
                t1
            } else {
                panic!("types stack is empty");
            };

            match sub_type {
                IntermediateType::IU8 => IU8::sub(builder, module),
                IntermediateType::IU16 => IU16::sub(builder, module),
                IntermediateType::IU32 => IU32::sub(builder, module),
                IntermediateType::IU64 => IU64::sub(builder, module),
                IntermediateType::IU128 => IU128::sub(builder, module, compilation_ctx),
                IntermediateType::IU256 => IU256::sub(builder, module, compilation_ctx),
                t => panic!("type stack error: trying to substract two {t:?}"),
            }

            types_stack.push(sub_type);
        }
        Bytecode::Mul => {
            let [t1, t2] = pop_n_from_stack(types_stack);
            assert_eq!(
                t1, t2,
                "types stack error: trying to multiply two different types {t1:?} {t2:?}"
            );

            match t1 {
                IntermediateType::IU8 => IU8::mul(builder, module),
                IntermediateType::IU16 => IU16::mul(builder, module),
                IntermediateType::IU32 => IU32::mul(builder, module),
                IntermediateType::IU64 => IU64::mul(builder, module),
                IntermediateType::IU128 => IU128::mul(builder, module, compilation_ctx),
                IntermediateType::IU256 => IU256::mul(builder, module, compilation_ctx),
                t => panic!("type stack error: trying to multiply two {t:?}"),
            }

            types_stack.push(t1);
        }
        Bytecode::Div => {
            let [t1, t2] = pop_n_from_stack(types_stack);
            assert_eq!(
                t1, t2,
                "types stack error: trying to divide two different types {t1:?} {t2:?}"
            );

            match t1 {
                IntermediateType::IU8 => IU8::div(builder),
                IntermediateType::IU16 => IU16::div(builder),
                IntermediateType::IU32 => IU32::div(builder),
                IntermediateType::IU64 => IU64::div(builder),
                IntermediateType::IU128 => IU128::div(builder, module, compilation_ctx),
                IntermediateType::IU256 => IU256::div(builder, module, compilation_ctx),
                t => panic!("type stack error: trying to divide two {t:?}"),
            }

            types_stack.push(t1);
        }
        Bytecode::Lt => {
            let [t1, t2] = pop_n_from_stack(types_stack);
            assert_eq!(
                t1, t2,
                "types stack error: trying to compare two different types {t1:?} {t2:?}"
            );

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
                _ => panic!("trying to compare two {t1:?}"),
            }

            types_stack.push(IntermediateType::IBool);
        }
        Bytecode::Le => {
            let [t1, t2] = pop_n_from_stack(types_stack);
            assert_eq!(
                t1, t2,
                "types stack error: trying to compare two different types {t1:?} {t2:?}"
            );

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
                _ => panic!("trying to compare two {t1:?}"),
            }

            types_stack.push(IntermediateType::IBool);
        }
        Bytecode::Gt => {
            let [t1, t2] = pop_n_from_stack(types_stack);
            assert_eq!(
                t1, t2,
                "types stack error: trying to compare two different types {t1:?} {t2:?}"
            );

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
                _ => panic!("trying to compare two {t1:?}"),
            }

            types_stack.push(IntermediateType::IBool);
        }
        Bytecode::Ge => {
            let [t1, t2] = pop_n_from_stack(types_stack);
            assert_eq!(
                t1, t2,
                "types stack error: trying to compare two different types {t1:?} {t2:?}"
            );

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
                _ => panic!("trying to compare two {t1:?}"),
            }

            types_stack.push(IntermediateType::IBool);
        }
        Bytecode::Mod => {
            let [t1, t2] = pop_n_from_stack(types_stack);
            assert_eq!(
                t1, t2,
                "types stack error: trying to mod two different types {t1:?} {t2:?}"
            );

            match t1 {
                IntermediateType::IU8 => IU8::remainder(builder),
                IntermediateType::IU16 => IU16::remainder(builder),
                IntermediateType::IU32 => IU32::remainder(builder),
                IntermediateType::IU64 => IU64::remainder(builder),
                IntermediateType::IU128 => IU128::remainder(builder, module, compilation_ctx),
                IntermediateType::IU256 => IU256::remainder(builder, module, compilation_ctx),
                t => panic!("type stack error: trying to mod two {t:?}"),
            }

            types_stack.push(t1);
        }
        Bytecode::Eq => {
            let [t1, t2] = pop_n_from_stack(types_stack);
            assert_eq!(
                t1, t2,
                "types stack error: trying to compare by equality two different types {t1:?} {t2:?}"
            );

            t1.load_equality_instructions(module, builder, compilation_ctx);

            types_stack.push(IntermediateType::IBool);
        }
        Bytecode::Neq => {
            let [t1, t2] = pop_n_from_stack(types_stack);
            assert_eq!(
                t1, t2,
                "types stack error: trying to compare by equality two different types {t1:?} {t2:?}"
            );

            t1.load_not_equality_instructions(module, builder, compilation_ctx);

            types_stack.push(IntermediateType::IBool);
        }
        Bytecode::Or => {
            pop_types_stack(types_stack, &IntermediateType::IBool).unwrap();
            pop_types_stack(types_stack, &IntermediateType::IBool).unwrap();
            builder.binop(BinaryOp::I32Or);
            types_stack.push(IntermediateType::IBool);
        }
        Bytecode::And => {
            pop_types_stack(types_stack, &IntermediateType::IBool).unwrap();
            pop_types_stack(types_stack, &IntermediateType::IBool).unwrap();
            builder.binop(BinaryOp::I32And);
            types_stack.push(IntermediateType::IBool);
        }
        Bytecode::Not => {
            pop_types_stack(types_stack, &IntermediateType::IBool).unwrap();
            builder.unop(UnaryOp::I32Eqz);
            types_stack.push(IntermediateType::IBool);
        }
        Bytecode::BitOr => {
            let t = if let (Some(t1), Some(t2)) = (types_stack.pop(), types_stack.pop()) {
                assert_eq!(
                    t1, t2,
                    "types stack error: trying two BitOr two different types {t1:?} {t2:?}"
                );
                t1
            } else {
                panic!("types stack is empty");
            };
            match t {
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
                _ => panic!("type stack error: trying to BitOr two {t:?}"),
            }
            types_stack.push(t);
        }
        Bytecode::BitAnd => {
            let t = if let (Some(t1), Some(t2)) = (types_stack.pop(), types_stack.pop()) {
                assert_eq!(
                    t1, t2,
                    "types stack error: trying two BitOr two different types {t1:?} {t2:?}"
                );
                t1
            } else {
                panic!("types stack is empty");
            };
            match t {
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
                _ => panic!("type stack error: trying to BitOr two {t:?}"),
            }
            types_stack.push(t);
        }
        Bytecode::Xor => {
            let t = if let (Some(t1), Some(t2)) = (types_stack.pop(), types_stack.pop()) {
                assert_eq!(
                    t1, t2,
                    "types stack error: trying two BitOr two different types {t1:?} {t2:?}"
                );
                t1
            } else {
                panic!("types stack is empty");
            };
            match t {
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
                _ => panic!("type stack error: trying to BitOr two {t:?}"),
            }
            types_stack.push(t);
        }
        Bytecode::Shl => {
            pop_types_stack(types_stack, &IntermediateType::IU8).unwrap();
            let t = types_stack.pop().unwrap();
            match t {
                IntermediateType::IU8 => IU8::bit_shift_left(builder, module),
                IntermediateType::IU16 => IU16::bit_shift_left(builder, module),
                IntermediateType::IU32 => IU32::bit_shift_left(builder, module),
                IntermediateType::IU64 => IU64::bit_shift_left(builder, module),
                IntermediateType::IU128 => IU128::bit_shift_left(builder, module, compilation_ctx),
                IntermediateType::IU256 => IU256::bit_shift_left(builder, module, compilation_ctx),
                t => panic!("type stack error: invalid type for Shl: {t:?}"),
            }
            types_stack.push(t);
        }
        Bytecode::Shr => {
            pop_types_stack(types_stack, &IntermediateType::IU8).unwrap();
            let t = types_stack.pop().unwrap();
            match t {
                IntermediateType::IU8 => IU8::bit_shift_right(builder, module),
                IntermediateType::IU16 => IU16::bit_shift_right(builder, module),
                IntermediateType::IU32 => IU32::bit_shift_right(builder, module),
                IntermediateType::IU64 => IU64::bit_shift_right(builder, module),
                IntermediateType::IU128 => IU128::bit_shift_right(builder, module, compilation_ctx),
                IntermediateType::IU256 => IU256::bit_shift_right(builder, module, compilation_ctx),
                t => panic!("type stack error: invalid type for Shr: {t:?}"),
            }
            types_stack.push(t);
        }
        Bytecode::Pack(struct_definition_index) => {
            let struct_ = compilation_ctx
                .get_struct_by_struct_definition_idx(struct_definition_index)
                .unwrap();

            bytecodes::structs::pack(struct_, module, builder, compilation_ctx, types_stack);

            types_stack.push(IntermediateType::IStruct(struct_definition_index.0));
        }
        Bytecode::PackGeneric(struct_definition_index) => {
            let struct_ = compilation_ctx
                .get_generic_struct_by_struct_definition_idx(struct_definition_index)
                .unwrap();

            bytecodes::structs::pack(&struct_, module, builder, compilation_ctx, types_stack);

            let idx = compilation_ctx
                .get_generic_struct_idx_by_struct_definition_idx(struct_definition_index);
            let types = compilation_ctx
                .get_generic_struct_types_instances(struct_definition_index)
                .unwrap();

            types_stack.push(IntermediateType::IGenericStructInstance(idx, types));
        }
        _ => panic!("Unsupported instruction: {:?}", instruction),
    }

    Ok(())
}

// Gets the IntermediateType for a given signature index
fn get_ir_for_signature_index(
    compilation_ctx: &CompilationContext,
    signature_index: SignatureIndex,
) -> IntermediateType {
    let signature_token = &compilation_ctx.module_signatures[signature_index.0 as usize].0;
    IntermediateType::try_from_signature_token(
        &signature_token[0],
        compilation_ctx.datatype_handles_map,
    )
    .unwrap()
}
