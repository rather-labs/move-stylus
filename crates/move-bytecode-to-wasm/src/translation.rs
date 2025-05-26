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
use walrus::ir::{BinaryOp, LoadKind, UnaryOp};
use walrus::{FunctionBuilder, Module};
use walrus::{
    FunctionId, InstrSeqBuilder, ValType,
    ir::{MemArg, StoreKind},
};

use crate::CompilationContext;

pub mod functions;
/// The types in this module represent an intermediate Rust representation of Move types
/// that is used to generate the WASM code.
pub mod intermediate_types;
pub mod table;

pub fn translate_function(
    module: &mut Module,
    index: usize,
    compilation_ctx: &CompilationContext,
    function_table: &FunctionTable,
) -> Result<FunctionId> {
    let entry = function_table
        .get(index)
        .ok_or(anyhow::anyhow!("index {index} not found in function table"))?;

    anyhow::ensure!(
        entry.function.move_code_unit.jump_tables.is_empty(),
        "Jump tables are not supported yet"
    );

    // Maps the types of the values contained in the stack at the moment of processing an
    // instruction.
    let mut types_stack = Vec::new();
    let mut function = FunctionBuilder::new(&mut module.types, &entry.params, &entry.results);
    let mut builder = function.func_body();

    for instruction in &entry.function.move_code_unit.code {
        map_bytecode_instruction(
            instruction,
            compilation_ctx,
            &mut builder,
            &entry.function,
            module,
            function_table,
            &mut types_stack,
        );
    }

    let function_id = function.finish(entry.function.arg_local_ids.clone(), &mut module.funcs);

    Ok(function_id)
}

fn map_bytecode_instruction(
    instruction: &Bytecode,
    compilation_ctx: &CompilationContext,
    builder: &mut InstrSeqBuilder,
    mapped_function: &MappedFunction,
    module: &mut Module,
    function_table: &FunctionTable,
    types_stack: &mut Vec<IntermediateType>,
) {
    match instruction {
        // Load a fixed constant
        Bytecode::LdConst(global_index) => {
            let constant = &compilation_ctx.constants[global_index.0 as usize];
            let mut data = constant.data.clone().into_iter();
            let constant_type = &constant.type_;
            let constant_type: IntermediateType = constant_type
                .try_into()
                // TODO: unwrap
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
            add_load_literal_heap_type_to_memory_instructions(
                module,
                builder,
                compilation_ctx,
                &literal.to_le_bytes(),
            );
            types_stack.push(IntermediateType::IU128);
        }
        Bytecode::LdU256(literal) => {
            add_load_literal_heap_type_to_memory_instructions(
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
                if let Err(e) = pop_types_stack(types_stack, argument) {
                    panic!("Called function signature arguments mismatch at index {i}: {e}");
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
            for return_type in return_types {
                types_stack.push(return_type.clone());
            }
        }
        // Locals
        Bytecode::StLoc(local_id) => {
            builder.local_set(mapped_function.local_variables[*local_id as usize]);

            pop_types_stack(
                types_stack,
                &mapped_function.local_variables_type[*local_id as usize],
            )
            .unwrap();
        }
        Bytecode::MoveLoc(local_id) => {
            // Values and references are loaded into a new variable
            // TODO: Find a way to ensure they will not be used again, the Move compiler should do the work for now
            builder.local_get(mapped_function.local_variables[*local_id as usize]);
            types_stack.push(mapped_function.local_variables_type[*local_id as usize].clone());
        }
        Bytecode::CopyLoc(local_id) => {
            mapped_function.local_variables_type[*local_id as usize].copy_loc_instructions(
                module,
                builder,
                compilation_ctx,
                mapped_function.local_variables[*local_id as usize],
            );
            types_stack.push(mapped_function.local_variables_type[*local_id as usize].clone());
        }
        Bytecode::VecPack(signature_index, num_elements) => {
            let inner =
                get_intermediate_type_for_signature_index(mapped_function, *signature_index);
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
                pop_types_stack(types_stack, &inner).unwrap();
                n -= 1;
            }

            types_stack.push(IntermediateType::IVector(Box::new(inner)));
        }
        Bytecode::ImmBorrowLoc(local_id) => {
            let local = mapped_function.local_variables[*local_id as usize];
            let local_type = &mapped_function.local_variables_type[*local_id as usize];

            local_type.add_imm_borrow_loc_instructions(module, builder, compilation_ctx, local);

            // Push the reference to the type into the types stack
            types_stack.push(IntermediateType::IRef(Box::new(local_type.clone())));
        }
        Bytecode::VecImmBorrow(signature_index) => {
            match (types_stack.pop(), types_stack.pop()) {
                (Some(IntermediateType::IU64), Some(IntermediateType::IRef(inner)))
                    if matches!(*inner, IntermediateType::IVector(_)) => {}
                (Some(t1), Some(t2)) => {
                    panic!("Expected IU64 and &vector<_>, got {t1:?} and {t2:?}")
                }
                _ => panic!("Type stack underflow"),
            }

            let inner_type =
                get_intermediate_type_for_signature_index(mapped_function, *signature_index);

            IVector::add_vec_imm_borrow_instructions(
                &inner_type,
                module,
                builder,
                compilation_ctx.memory_id,
            );

            // Push &T onto the WASM type stack
            types_stack.push(IntermediateType::IRef(Box::new(inner_type)));
        }

        Bytecode::VecLen(signature_index) => {
            let expected_elem_type =
                get_intermediate_type_for_signature_index(mapped_function, *signature_index);

            let expected_type = IntermediateType::IVector(Box::new(expected_elem_type.clone()));

            match types_stack.pop() {
                Some(IntermediateType::IRef(actual_type)) => {
                    if *actual_type != expected_type {
                        panic!(
                            "Type mismatch: expected &vector<{:?}> but got &{:?}",
                            expected_elem_type, actual_type
                        );
                    }
                }
                Some(t) => panic!("Expected &vector<_>, got {:?}", t),
                None => panic!("Type stack underflow"),
            }

            builder
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
            let ref_type = types_stack
                .pop()
                .expect("ReadRef expects a reference on the stack");

            match ref_type {
                IntermediateType::IRef(inner) => {
                    // Now call directly on the inner type
                    inner.add_read_ref_instructions(builder, compilation_ctx.memory_id);

                    // And push the inner type into the stack
                    types_stack.push(*inner);
                }
                _ => panic!("ReadRef expected a IRef type but got: {:?}", ref_type),
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
        _ => panic!("Unsupported instruction: {:?}", instruction),
    }
}

fn add_load_literal_heap_type_to_memory_instructions(
    module: &mut Module,
    builder: &mut InstrSeqBuilder,
    compilation_ctx: &CompilationContext,
    bytes: &[u8],
) {
    let pointer = module.locals.add(ValType::I32);

    builder.i32_const(bytes.len() as i32);
    builder.call(compilation_ctx.allocator);
    builder.local_set(pointer);

    let mut offset = 0;

    while offset < bytes.len() {
        builder.local_get(pointer);
        builder.i64_const(i64::from_le_bytes(
            bytes[offset..offset + 8].try_into().unwrap(),
        ));
        builder.store(
            compilation_ctx.memory_id,
            StoreKind::I64 { atomic: false },
            MemArg {
                align: 0,
                offset: offset as u32,
            },
        );

        offset += 8;
    }

    builder.local_get(pointer);
}

fn get_intermediate_type_for_signature_index(
    mapped_function: &MappedFunction,
    signature_index: SignatureIndex,
) -> IntermediateType {
    let tokens = &mapped_function.move_module_signatures[signature_index.0 as usize].0;
    assert_eq!(
        tokens.len(),
        1,
        "Expected signature to have exactly 1 token for VecPack"
    );
    (&tokens[0]).try_into().unwrap()
}

fn pop_types_stack(
    types_stack: &mut Vec<IntermediateType>,
    expected_type: &IntermediateType,
) -> Result<()> {
    let Some(ty) = types_stack.pop() else {
        anyhow::bail!(
            "error popping types stack: expected {expected_type:?} but types stack is empty"
        );
    };
    anyhow::ensure!(
        ty == *expected_type,
        "expected {expected_type:?} and found {ty:?}"
    );
    Ok(())
}
