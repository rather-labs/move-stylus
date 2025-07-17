use crate::CompilationContext;
use crate::compilation_context::CompilationContextError;
use crate::translation::MappedFunction;
use crate::translation::TranslationError;
use crate::translation::bytecodes::vectors;
use crate::translation::intermediate_types::IntermediateType;
use move_binary_format::file_format::Bytecode;
use walrus::ValType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypesStack(pub Vec<IntermediateType>);

type Result<T> = std::result::Result<T, TypesStackError>;

impl TypesStack {
    pub fn new() -> Self {
        TypesStack(Vec::new())
    }

    pub fn push(&mut self, item: IntermediateType) {
        self.0.push(item)
    }

    pub fn pop(&mut self) -> Result<IntermediateType> {
        self.0.pop().ok_or(TypesStackError::EmptyStack)
    }

    pub fn append(&mut self, items: &[IntermediateType]) {
        self.0.extend_from_slice(items);
    }

    pub fn pop_expecting(&mut self, expected_type: &IntermediateType) -> Result<()> {
        let Ok(ty) = self.pop() else {
            return Err(TypesStackError::EmptyStackExpecting {
                expected: expected_type.clone(),
            });
        };

        if ty != *expected_type {
            return Err(TypesStackError::TypeMismatch {
                expected: expected_type.clone(),
                found: ty,
            });
        }

        Ok(())
    }

    pub fn pop_n_from_stack<const N: usize>(&mut self) -> Result<[IntermediateType; N]> {
        // We use IU8 as placeholder, it gets replaced on the for loop
        let mut res = [const { IntermediateType::IU8 }; N];
        #[allow(clippy::needless_range_loop)]
        for i in 0..N {
            if let Ok(t) = self.pop() {
                res[i] = t;
            } else {
                return Err(TypesStackError::ExpectedNElements(N));
            }
        }

        Ok(res)
    }

    pub fn to_val_types(&self) -> Vec<ValType> {
        self.0.iter().map(ValType::from).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn pack_struct(&mut self, struct_index: usize, fields: &[IntermediateType]) -> Result<()> {
        for expected_type in fields.iter().rev() {
            let found_type = self.pop()?;

            match (&found_type, expected_type) {
                (a, b) if a == b => match b {
                    IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                        return Err(TypesStackError::Translation(Box::new(
                            TranslationError::FoundReferenceInsideStruct {
                                struct_index: struct_index.try_into().unwrap(),
                            },
                        )));
                    }
                    IntermediateType::ITypeParameter(index) => {
                        return Err(TypesStackError::Translation(Box::new(
                            TranslationError::FoundTypeParameterInsideStruct {
                                struct_index: struct_index.try_into().unwrap(),
                                type_parameter_index: *index,
                            },
                        )));
                    }
                    _ => {}
                },
                _ => {
                    return Err(TypesStackError::Translation(Box::new(
                        TranslationError::TypeMismatch {
                            expected: expected_type.clone(),
                            found: found_type,
                        },
                    )));
                }
            }
        }

        Ok(())
    }

    fn unpack_struct(&mut self, struct_index: usize, fields: &[IntermediateType]) -> Result<()> {
        for field in fields {
            match field {
                IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                    return Err(TypesStackError::Translation(Box::new(
                        TranslationError::FoundReferenceInsideStruct {
                            struct_index: struct_index.try_into().unwrap(),
                        },
                    )));
                }
                IntermediateType::ITypeParameter(index) => {
                    return Err(TypesStackError::Translation(Box::new(
                        TranslationError::FoundTypeParameterInsideStruct {
                            struct_index: struct_index.try_into().unwrap(),
                            type_parameter_index: *index,
                        },
                    )));
                }
                _ => {
                    self.push(field.clone());
                }
            }
        }
        Ok(())
    }

    // TODO: check error handling
    // TODO: extract repeated logic into functions
    pub fn process_instruction(
        &mut self,
        instruction: &Bytecode,
        compilation_ctx: &CompilationContext,
        mapped_function: &MappedFunction,
    ) -> Result<()> {
        match instruction {
            // Load a fixed constant
            Bytecode::LdConst(global_index) => {
                let constant = &compilation_ctx.root_module_data.constants[global_index.0 as usize];
                let constant_type = &constant.type_;
                let constant_type: IntermediateType = IntermediateType::try_from_signature_token(
                    constant_type,
                    &compilation_ctx.root_module_data.datatype_handles_map,
                )?;

                self.push(constant_type);
            }
            // Load literals
            Bytecode::LdFalse => {
                self.push(IntermediateType::IBool);
            }
            Bytecode::LdTrue => {
                self.push(IntermediateType::IBool);
            }
            Bytecode::LdU8(_) => {
                self.push(IntermediateType::IU8);
            }
            Bytecode::LdU16(_) => {
                self.push(IntermediateType::IU16);
            }
            Bytecode::LdU32(_) => {
                self.push(IntermediateType::IU32);
            }
            Bytecode::LdU64(_) => {
                self.push(IntermediateType::IU64);
            }
            Bytecode::LdU128(_) => {
                self.push(IntermediateType::IU128);
            }
            Bytecode::LdU256(_) => {
                self.push(IntermediateType::IU256);
            }
            // Function calls
            Bytecode::Call(function_handle_index) => {
                // Consume from the types stack the arguments that will be used by the function call
                let arguments = &compilation_ctx.root_module_data.functions_arguments
                    [function_handle_index.0 as usize];
                for argument in arguments.iter().rev() {
                    self.pop_expecting(argument)?;
                }

                // Insert in the stack types the types returned by the function (if any)
                let return_types = &compilation_ctx.root_module_data.functions_returns
                    [function_handle_index.0 as usize];
                self.append(return_types);
            }
            // Locals
            Bytecode::StLoc(local_id) => {
                let local_type = &mapped_function.function_locals_ir[*local_id as usize];
                self.pop_expecting(local_type)?;
            }
            Bytecode::MoveLoc(local_id) => {
                let local_type = mapped_function.function_locals_ir[*local_id as usize].clone();
                self.push(local_type);
            }
            Bytecode::CopyLoc(local_id) => {
                let local_type = mapped_function.function_locals_ir[*local_id as usize].clone();
                self.push(local_type);
            }
            Bytecode::ImmBorrowLoc(local_id) => {
                let local_type = &mapped_function.function_locals_ir[*local_id as usize];
                self.push(IntermediateType::IRef(Box::new(local_type.clone())));
            }
            Bytecode::MutBorrowLoc(local_id) => {
                let local_type = &mapped_function.function_locals_ir[*local_id as usize];
                self.push(IntermediateType::IMutRef(Box::new(local_type.clone())));
            }
            Bytecode::ImmBorrowField(field_id) => {
                let struct_ = compilation_ctx
                    .get_struct_by_field_handle_idx(field_id)
                    .map_err(TypesStackError::Compilation)?;
                self.pop_expecting(&IntermediateType::IRef(Box::new(
                    IntermediateType::IStruct(struct_.index()),
                )))?;

                let Some(field_type) = struct_.fields_types.get(field_id) else {
                    panic!(
                        "{field_id} not found in {}",
                        struct_.struct_definition_index
                    )
                };

                self.push(IntermediateType::IRef(Box::new(field_type.clone())));
            }
            Bytecode::ImmBorrowFieldGeneric(field_id) => {
                let (struct_field_id, instantiation_types) = compilation_ctx
                    .root_module_data
                    .instantiated_fields_to_generic_fields
                    .get(field_id)
                    .unwrap();

                let instantiation_types: Vec<_> = instantiation_types
                    .iter()
                    .map(|t| {
                        IntermediateType::try_from_signature_token(
                            t,
                            &compilation_ctx.root_module_data.datatype_handles_map,
                        )
                        .map_err(TypesStackError::from)
                    })
                    .collect::<Result<_>>()?;

                let struct_ = if let Ok(struct_) =
                    compilation_ctx.get_generic_struct_by_field_handle_idx(field_id)
                {
                    struct_
                } else {
                    let generic_stuct = compilation_ctx
                        .get_struct_by_field_handle_idx(struct_field_id)
                        .map_err(TypesStackError::Compilation)?;
                    generic_stuct.instantiate(&instantiation_types)
                };

                // Check if in the types stack we have the correct type
                self.pop_expecting(&IntermediateType::IRef(Box::new(
                    IntermediateType::IGenericStructInstance(struct_.index(), instantiation_types),
                )))?;

                let Some(field_type) = struct_.fields_types.get(struct_field_id) else {
                    panic!(
                        "{field_id} not found in {}",
                        struct_.struct_definition_index
                    )
                };

                self.push(IntermediateType::IRef(Box::new(field_type.clone())));
            }
            Bytecode::MutBorrowField(field_id) => {
                let struct_ = compilation_ctx
                    .get_struct_by_field_handle_idx(field_id)
                    .map_err(TypesStackError::Compilation)?;

                // Check if in the types stack we have the correct type
                self.pop_expecting(&IntermediateType::IMutRef(Box::new(
                    IntermediateType::IStruct(struct_.index()),
                )))?;

                let Some(field_type) = struct_.fields_types.get(field_id) else {
                    panic!(
                        "{field_id:?} not found in {}",
                        struct_.struct_definition_index
                    )
                };

                self.push(IntermediateType::IMutRef(Box::new(field_type.clone())));
            }
            Bytecode::MutBorrowFieldGeneric(field_id) => {
                let (struct_field_id, instantiation_types) = compilation_ctx
                    .root_module_data
                    .instantiated_fields_to_generic_fields
                    .get(field_id)
                    .unwrap();

                let instantiation_types: Vec<_> = instantiation_types
                    .iter()
                    .map(|t| {
                        IntermediateType::try_from_signature_token(
                            t,
                            &compilation_ctx.root_module_data.datatype_handles_map,
                        )
                        .map_err(TypesStackError::from)
                    })
                    .collect::<Result<_>>()?;

                let struct_ = if let Ok(struct_) =
                    compilation_ctx.get_generic_struct_by_field_handle_idx(field_id)
                {
                    struct_
                } else {
                    let generic_stuct = compilation_ctx
                        .get_struct_by_field_handle_idx(struct_field_id)
                        .map_err(|e| TypesStackError::Compilation(e))?;
                    generic_stuct.instantiate(&instantiation_types)
                };

                // Check if in the types stack we have the correct type
                self.pop_expecting(&IntermediateType::IMutRef(Box::new(
                    IntermediateType::IGenericStructInstance(struct_.index(), instantiation_types),
                )))?;

                let Some(field_type) = struct_.fields_types.get(struct_field_id) else {
                    panic!(
                        "{field_id:?} not found in {}",
                        struct_.struct_definition_index
                    )
                };

                self.push(IntermediateType::IMutRef(Box::new(field_type.clone())));
            }
            // Vector instructions
            Bytecode::VecImmBorrow(signature_index) => {
                let [t1, t2] = self.pop_n_from_stack()?;

                match_types!(
                    (IntermediateType::IU64, "u64", t1),
                    (IntermediateType::IRef(ref_inner), "vector reference", t2),
                    (IntermediateType::IVector(vec_inner), "vector", *ref_inner)
                );

                let expected_vec_inner =
                    vectors::get_inner_type_from_signature(signature_index, compilation_ctx)
                        .map_err(|e| TypesStackError::Translation(Box::new(e)))?;

                if *vec_inner != expected_vec_inner {
                    return Err(TypesStackError::TypeMismatch {
                        expected: expected_vec_inner,
                        found: *vec_inner,
                    });
                }

                self.push(IntermediateType::IRef(Box::new(*vec_inner)));
            }
            Bytecode::VecMutBorrow(signature_index) => {
                let [t1, t2] = self.pop_n_from_stack()?;

                match_types!(
                    (IntermediateType::IU64, "u64", t1),
                    (
                        IntermediateType::IMutRef(ref_inner),
                        "mutable vector reference",
                        t2
                    ),
                    (IntermediateType::IVector(vec_inner), "vector", *ref_inner)
                );

                let expected_vec_inner =
                    vectors::get_inner_type_from_signature(signature_index, compilation_ctx)
                        .map_err(|e| TypesStackError::Translation(Box::new(e)))?;

                if *vec_inner != expected_vec_inner {
                    return Err(TypesStackError::TypeMismatch {
                        expected: expected_vec_inner,
                        found: *vec_inner,
                    });
                }

                self.push(IntermediateType::IMutRef(Box::new(*vec_inner)));
            }
            Bytecode::VecPack(signature_index, num_elements) => {
                let inner =
                    vectors::get_inner_type_from_signature(signature_index, compilation_ctx)
                        .map_err(|e| TypesStackError::Translation(Box::new(e)))?;

                // Remove the packing values from types stack and check if the types are correct
                let mut n = *num_elements as usize;
                while n > 0 {
                    self.pop_expecting(&inner)?;
                    n -= 1;
                }

                self.push(IntermediateType::IVector(Box::new(inner)));
            }
            Bytecode::VecPopBack(signature_index) => {
                let ty = self.pop()?;

                match_types!(
                    (
                        IntermediateType::IMutRef(ref_inner),
                        "mutable vector reference",
                        ty
                    ),
                    (IntermediateType::IVector(vec_inner), "vector", *ref_inner)
                );

                let expected_vec_inner =
                    vectors::get_inner_type_from_signature(signature_index, compilation_ctx)
                        .map_err(|e| TypesStackError::Translation(Box::new(e)))?;

                if *vec_inner != expected_vec_inner {
                    return Err(TypesStackError::TypeMismatch {
                        expected: expected_vec_inner,
                        found: *vec_inner,
                    });
                }

                self.push(*vec_inner);
            }
            Bytecode::VecPushBack(signature_index) => {
                let [elem_ty, ref_ty] = self.pop_n_from_stack()?;

                match_types!(
                    (
                        IntermediateType::IMutRef(mut_inner),
                        "mutable vector reference",
                        ref_ty
                    ),
                    (IntermediateType::IVector(vec_inner), "vector", *mut_inner)
                );

                let expected_elem_type =
                    vectors::get_inner_type_from_signature(signature_index, compilation_ctx)
                        .map_err(|e| TypesStackError::Translation(Box::new(e)))?;

                if *vec_inner != expected_elem_type {
                    return Err(TypesStackError::TypeMismatch {
                        expected: expected_elem_type,
                        found: *vec_inner,
                    });
                }

                if elem_ty != expected_elem_type {
                    return Err(TypesStackError::TypeMismatch {
                        expected: expected_elem_type,
                        found: elem_ty,
                    });
                }
            }
            Bytecode::VecSwap(signature_index) => {
                let [id2_ty, id1_ty, ref_ty] = self.pop_n_from_stack()?;

                match_types!(
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
                    vectors::get_inner_type_from_signature(signature_index, compilation_ctx)
                        .map_err(|e| TypesStackError::Translation(Box::new(e)))?;

                if *vec_inner != expected_vec_inner {
                    return Err(TypesStackError::TypeMismatch {
                        expected: expected_vec_inner,
                        found: *vec_inner,
                    });
                }
            }
            Bytecode::VecLen(signature_index) => {
                let elem_ir_type =
                    vectors::get_inner_type_from_signature(signature_index, compilation_ctx)
                        .map_err(|e| TypesStackError::Translation(Box::new(e)))?;

                self.pop_expecting(&IntermediateType::IRef(Box::new(
                    IntermediateType::IVector(Box::new(elem_ir_type)),
                )))?;

                self.push(IntermediateType::IU64);
            }
            Bytecode::ReadRef => {
                let ref_type = self.pop()?;

                match_types!((
                    (IntermediateType::IRef(inner) | IntermediateType::IMutRef(inner)),
                    "IRef or IMutRef",
                    ref_type
                ));

                self.push(*inner);
            }
            Bytecode::WriteRef => {
                let [iref, _] = self.pop_n_from_stack()?;

                match_types!((IntermediateType::IMutRef(_), "IMutRef", iref));
            }
            Bytecode::FreezeRef => {
                let ref_type = self.pop()?;

                match_types!((
                    IntermediateType::IMutRef(inner),
                    "mutable reference",
                    ref_type
                ));

                self.push(IntermediateType::IRef(inner.clone()));
            }
            Bytecode::Pop => {
                self.pop()?;
            }
            // TODO: ensure this is the last instruction in the move code
            Bytecode::Ret => {
                // We dont pop the return values from the stack, we just check if the types match
                assert!(
                    self.0.ends_with(&mapped_function.signature.returns),
                    "types stack does not match function return types"
                );
            }
            Bytecode::CastU8 => {
                self.pop()?;
                self.push(IntermediateType::IU8);
            }
            Bytecode::CastU16 => {
                self.pop()?;
                self.push(IntermediateType::IU16);
            }
            Bytecode::CastU32 => {
                self.pop()?;
                self.push(IntermediateType::IU32);
            }
            Bytecode::CastU64 => {
                self.pop()?;
                self.push(IntermediateType::IU64);
            }
            Bytecode::CastU128 => {
                self.pop()?;
                self.push(IntermediateType::IU128);
            }
            Bytecode::CastU256 => {
                self.pop()?;
                self.push(IntermediateType::IU256);
            }
            Bytecode::Add => {
                let [t1, t2] = self.pop_n_from_stack()?;
                if t1 != t2 {
                    return Err(TypesStackError::OperationTypeMismatch {
                        operand1: t1,
                        operand2: t2,
                        operation: Bytecode::Add,
                    });
                }

                self.push(t2);
            }
            Bytecode::Sub => {
                let [t1, t2] = self.pop_n_from_stack()?;
                if t1 != t2 {
                    return Err(TypesStackError::OperationTypeMismatch {
                        operand1: t1,
                        operand2: t2,
                        operation: Bytecode::Sub,
                    });
                }

                self.push(t2);
            }
            Bytecode::Mul => {
                let [t1, t2] = self.pop_n_from_stack()?;
                if t1 != t2 {
                    return Err(TypesStackError::OperationTypeMismatch {
                        operand1: t1,
                        operand2: t2,
                        operation: Bytecode::Mul,
                    });
                }

                self.push(t2);
            }
            Bytecode::Div => {
                let [t1, t2] = self.pop_n_from_stack()?;
                if t1 != t2 {
                    return Err(TypesStackError::OperationTypeMismatch {
                        operand1: t1,
                        operand2: t2,
                        operation: Bytecode::Div,
                    });
                }

                self.push(t2);
            }
            Bytecode::Lt => {
                let [t1, t2] = self.pop_n_from_stack()?;
                if t1 != t2 {
                    return Err(TypesStackError::OperationTypeMismatch {
                        operand1: t1,
                        operand2: t2,
                        operation: Bytecode::Lt,
                    });
                }

                self.push(IntermediateType::IBool);
            }
            Bytecode::Le => {
                let [t1, t2] = self.pop_n_from_stack()?;
                if t1 != t2 {
                    return Err(TypesStackError::OperationTypeMismatch {
                        operand1: t1,
                        operand2: t2,
                        operation: Bytecode::Le,
                    });
                }

                self.push(IntermediateType::IBool);
            }
            Bytecode::Gt => {
                let [t1, t2] = self.pop_n_from_stack()?;
                if t1 != t2 {
                    return Err(TypesStackError::OperationTypeMismatch {
                        operand1: t1,
                        operand2: t2,
                        operation: Bytecode::Gt,
                    });
                }

                self.push(IntermediateType::IBool);
            }
            Bytecode::Ge => {
                let [t1, t2] = self.pop_n_from_stack()?;
                if t1 != t2 {
                    return Err(TypesStackError::OperationTypeMismatch {
                        operand1: t1,
                        operand2: t2,
                        operation: Bytecode::Ge,
                    });
                }

                self.push(IntermediateType::IBool);
            }
            Bytecode::Mod => {
                let [t1, t2] = self.pop_n_from_stack()?;
                if t1 != t2 {
                    return Err(TypesStackError::OperationTypeMismatch {
                        operand1: t1,
                        operand2: t2,
                        operation: Bytecode::Mod,
                    });
                }

                self.push(t2);
            }
            Bytecode::Eq => {
                let [t1, t2] = self.pop_n_from_stack()?;
                if t1 != t2 {
                    return Err(TypesStackError::OperationTypeMismatch {
                        operand1: t1,
                        operand2: t2,
                        operation: Bytecode::Eq,
                    });
                }

                self.push(IntermediateType::IBool);
            }
            Bytecode::Neq => {
                let [t1, t2] = self.pop_n_from_stack()?;
                if t1 != t2 {
                    return Err(TypesStackError::OperationTypeMismatch {
                        operand1: t1,
                        operand2: t2,
                        operation: Bytecode::Neq,
                    });
                }

                self.push(IntermediateType::IBool);
            }
            Bytecode::Or => {
                self.pop_expecting(&IntermediateType::IBool)?;
                self.pop_expecting(&IntermediateType::IBool)?;
                self.push(IntermediateType::IBool);
            }
            Bytecode::And => {
                self.pop_expecting(&IntermediateType::IBool)?;
                self.pop_expecting(&IntermediateType::IBool)?;
                self.push(IntermediateType::IBool);
            }
            Bytecode::Not => {
                self.pop_expecting(&IntermediateType::IBool)?;
                self.push(IntermediateType::IBool);
            }
            Bytecode::BitOr => {
                let [t1, t2] = self.pop_n_from_stack()?;
                if t1 != t2 {
                    return Err(TypesStackError::OperationTypeMismatch {
                        operand1: t1,
                        operand2: t2,
                        operation: Bytecode::BitOr,
                    });
                }

                self.push(t2);
            }
            Bytecode::BitAnd => {
                let [t1, t2] = self.pop_n_from_stack()?;
                if t1 != t2 {
                    return Err(TypesStackError::OperationTypeMismatch {
                        operand1: t1,
                        operand2: t2,
                        operation: Bytecode::BitAnd,
                    });
                }

                self.push(t2);
            }
            Bytecode::Xor => {
                let [t1, t2] = self.pop_n_from_stack()?;
                if t1 != t2 {
                    return Err(TypesStackError::OperationTypeMismatch {
                        operand1: t1,
                        operand2: t2,
                        operation: Bytecode::BitOr,
                    });
                }

                self.push(t1);
            }
            Bytecode::Shl => {
                self.pop_expecting(&IntermediateType::IU8)?;
                let t = self.pop()?;
                self.push(t);
            }
            Bytecode::Shr => {
                self.pop_expecting(&IntermediateType::IU8)?;
                let t = self.pop()?;
                self.push(t);
            }
            Bytecode::Pack(struct_definition_index) => {
                let struct_ =
                    compilation_ctx.get_struct_by_struct_definition_idx(struct_definition_index)?;

                self.pack_struct(struct_.index().into(), &struct_.fields)?;

                self.push(IntermediateType::IStruct(struct_definition_index.0));
            }
            Bytecode::PackGeneric(struct_definition_index) => {
                let struct_ = compilation_ctx
                    .get_generic_struct_by_struct_definition_idx(struct_definition_index)?;

                let idx = compilation_ctx
                    .get_generic_struct_idx_by_struct_definition_idx(struct_definition_index);

                self.pack_struct(idx.into(), &struct_.fields)?;

                let types =
                    compilation_ctx.get_generic_struct_types_instances(struct_definition_index)?;

                self.push(IntermediateType::IGenericStructInstance(idx, types));
            }
            Bytecode::Unpack(struct_definition_index) => {
                self.pop_expecting(&IntermediateType::IStruct(struct_definition_index.0))?;

                let struct_ =
                    compilation_ctx.get_struct_by_struct_definition_idx(struct_definition_index)?;

                self.unpack_struct(struct_.index().into(), &struct_.fields)?;
            }
            Bytecode::UnpackGeneric(struct_definition_index) => {
                let idx = compilation_ctx
                    .get_generic_struct_idx_by_struct_definition_idx(struct_definition_index);

                let types =
                    compilation_ctx.get_generic_struct_types_instances(struct_definition_index)?;

                self.pop_expecting(&IntermediateType::IGenericStructInstance(idx, types))?;

                let struct_ = compilation_ctx
                    .get_generic_struct_by_struct_definition_idx(struct_definition_index)?;

                self.unpack_struct(struct_.index().into(), &struct_.fields)?;
            }
            // Control flows
            Bytecode::BrTrue(_) | Bytecode::BrFalse(_) => {
                self.pop_expecting(&IntermediateType::IBool)?;
            }
            Bytecode::Branch(_) => (),
            Bytecode::PackVariant(index) => {
                let enum_ = compilation_ctx.get_enum_by_variant_handle_idx(index)?;
                let index_inside_enum =
                    compilation_ctx.get_variant_position_by_variant_handle_idx(index)?;

                for pack_type in enum_.variants[index_inside_enum as usize]
                    .fields
                    .iter()
                    .rev()
                {
                    if self.pop()? != *pack_type {
                        return Err(TypesStackError::TypeMismatch {
                            expected: pack_type.clone(),
                            found: self.pop()?,
                        });
                    }
                }
                self.push(IntermediateType::IEnum(enum_.index));
            }
            b => Err(TypesStackError::Translation(Box::new(
                TranslationError::UnsupportedOperation {
                    operation: b.clone(),
                },
            )))?,
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TypesStackError {
    #[error("expected {expected:?} but types stack is empty")]
    EmptyStackExpecting { expected: IntermediateType },

    #[error("types stack is empty")]
    EmptyStack,

    #[error("expected {0} but types stack is empty")]
    ExpectedNElements(usize),

    #[error("expected {expected:?} but found {found:?}")]
    TypeMismatch {
        expected: IntermediateType,
        found: IntermediateType,
    },

    #[error("expected {expected:?} but found {found:?}")]
    MatchError {
        expected: &'static str,
        found: IntermediateType,
    },

    #[error(
        "unable to perform \"{operation:?}\" on types {operand1:?} and {operand2:?}, expected the same type on types stack"
    )]
    OperationTypeMismatch {
        operand1: IntermediateType,
        operand2: IntermediateType,
        operation: Bytecode,
    },

    #[error("External error: {0}")]
    External(#[from] anyhow::Error),

    #[error("Translation error: {0}")]
    Translation(Box<TranslationError>),

    #[error("Compilation error: {0}")]
    Compilation(#[from] CompilationContextError),
}

macro_rules! match_types {
    ($(($expected_pattern: pat, $expected_type: expr, $variable: expr)),*) => {
        $(
            let $expected_pattern = $variable else {
                return Err($crate::translation::types_stack::TypesStackError::MatchError {
                    expected: $expected_type,
                    found: $variable,
                })?;
            };
        )*
    };
}

pub(crate) use match_types;
