pub mod address;
pub mod boolean;
pub mod enums;
pub mod error;
pub mod heap_integers;
pub mod reference;
pub mod signer;
pub mod simple_integers;
pub mod structs;
pub(crate) mod user_type_fields;
pub mod vector;

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use crate::{
    CompilationContext, UserDefinedType,
    compilation_context::{ModuleData, ModuleId, module_data::Address},
    hasher::get_hasher,
    runtime::RuntimeFunction,
    wasm_builder_extensions::WasmBuilderExtension,
};

use error::IntermediateTypeError;
use move_binary_format::file_format::{DatatypeHandleIndex, Signature, SignatureToken};

use address::IAddress;
use boolean::IBool;
use heap_integers::{IU128, IU256};
use simple_integers::{IU8, IU16, IU32, IU64};
use vector::IVector;

use walrus::{
    InstrSeqBuilder, LocalId, MemoryId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use super::TranslationError;

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub enum VmHandledStruct {
    // Can be either a UID or NamedId
    StorageId {
        /// Wrapping struct's module id
        parent_module_id: ModuleId,
        /// Wrapping struct's index
        parent_index: u16,
        /// If the wrapping struct is concrete this field will be None
        /// Otherwise it will contain the list of the type instantiatons
        instance_types: Option<Vec<IntermediateType>>,
    },
    None,
}

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub enum IntermediateType {
    IBool,
    IU8,
    IU16,
    IU32,
    IU64,
    IU128,
    IU256,
    IAddress,
    ISigner,
    IVector(Box<IntermediateType>),
    IRef(Box<IntermediateType>),
    IMutRef(Box<IntermediateType>),

    /// Type parameter, used for generic enums and structs
    /// The u16 is the index of the type parameter in the signature
    ITypeParameter(u16),

    /// Intermediate struct representation
    ///
    /// The u16 is the struct's index in the compilation context's vector of declared structs
    IStruct {
        module_id: ModuleId,
        index: u16,
        vm_handled_struct: VmHandledStruct,
    },

    /// The usize is the index of the generic struct.
    /// The Vec<IntermediateType> is the list of types we are going to instantiate the generic
    /// struct with.
    IGenericStructInstance {
        module_id: ModuleId,
        index: u16,
        types: Vec<IntermediateType>,
        vm_handled_struct: VmHandledStruct,
    },

    /// Intermediate enum representation
    ///
    /// The module_id is the module id of the enum.
    /// The index is the enum's index in the compilation context.
    IEnum {
        module_id: ModuleId,
        index: u16,
    },

    /// Intermediate generic enum instance representation
    ///
    /// The first u16 is the enum's index in the compilation context.
    /// The Vec<IntermediateType> is the list of types we are going to instantiate the generic
    /// enum with.
    IGenericEnumInstance {
        module_id: ModuleId,
        index: u16,
        types: Vec<IntermediateType>,
    },
}

impl IntermediateType {
    /// Returns the size in bytes, that this type needs in memory to be stored
    pub fn stack_data_size(&self) -> Result<u32, IntermediateTypeError> {
        let size = match self {
            IntermediateType::IU64 => 8,
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::IAddress
            | IntermediateType::ISigner
            | IntermediateType::IVector(_)
            | IntermediateType::IRef(_)
            | IntermediateType::IMutRef(_)
            | IntermediateType::IStruct { .. }
            | IntermediateType::IGenericStructInstance { .. }
            | IntermediateType::IEnum { .. }
            | IntermediateType::IGenericEnumInstance { .. } => 4,
            IntermediateType::ITypeParameter(_) => {
                return Err(IntermediateTypeError::FoundTypeParameter);
            }
        };

        Ok(size)
    }

    pub fn try_from_signature_token(
        value: &SignatureToken,
        handles_map: &HashMap<DatatypeHandleIndex, UserDefinedType>,
    ) -> Result<Self, IntermediateTypeError> {
        match value {
            SignatureToken::Bool => Ok(Self::IBool),
            SignatureToken::U8 => Ok(Self::IU8),
            SignatureToken::U16 => Ok(Self::IU16),
            SignatureToken::U32 => Ok(Self::IU32),
            SignatureToken::U64 => Ok(Self::IU64),
            SignatureToken::U128 => Ok(Self::IU128),
            SignatureToken::U256 => Ok(Self::IU256),
            SignatureToken::Address => Ok(Self::IAddress),
            SignatureToken::Signer => Ok(Self::ISigner),
            SignatureToken::Vector(token) => {
                let itoken = Self::try_from_signature_token(token.as_ref(), handles_map)?;
                Ok(IntermediateType::IVector(Box::new(itoken)))
            }
            SignatureToken::Reference(token) => {
                let itoken = Self::try_from_signature_token(token.as_ref(), handles_map)?;
                Ok(IntermediateType::IRef(Box::new(itoken)))
            }
            SignatureToken::MutableReference(token) => {
                let itoken = Self::try_from_signature_token(token.as_ref(), handles_map)?;
                Ok(IntermediateType::IMutRef(Box::new(itoken)))
            }
            SignatureToken::Datatype(datatype_index) => {
                if let Some(udt) = handles_map.get(datatype_index) {
                    Ok(match udt {
                        UserDefinedType::Struct { module_id, index } => IntermediateType::IStruct {
                            module_id: module_id.clone(),
                            index: *index,
                            vm_handled_struct: VmHandledStruct::None,
                        },
                        UserDefinedType::Enum { module_id, index } => IntermediateType::IEnum {
                            module_id: module_id.clone(),
                            index: *index,
                        },
                    })
                } else {
                    Err(IntermediateTypeError::UserDefinedTypeNotFound(
                        *datatype_index,
                    ))
                }
            }
            SignatureToken::DatatypeInstantiation(index) => {
                if let Some(udt) = handles_map.get(&index.0) {
                    let types = index
                        .1
                        .iter()
                        .map(|t| Self::try_from_signature_token(t, handles_map))
                        .collect::<Result<Vec<IntermediateType>, IntermediateTypeError>>()?;

                    Ok(match udt {
                        UserDefinedType::Struct { module_id, index } => {
                            IntermediateType::IGenericStructInstance {
                                module_id: module_id.clone(),
                                index: *index,
                                types,
                                vm_handled_struct: VmHandledStruct::None,
                            }
                        }
                        UserDefinedType::Enum { module_id, index } => {
                            IntermediateType::IGenericEnumInstance {
                                module_id: module_id.clone(),
                                index: *index,
                                types: types.clone(),
                            }
                        }
                    })
                } else {
                    Err(IntermediateTypeError::UserDefinedTypeNotFound(index.0))
                }
            }
            SignatureToken::TypeParameter(index) => Ok(IntermediateType::ITypeParameter(*index)),
        }
    }

    /// Adds the instructions to load the constant into the local variable
    /// Pops the first n elements from `bytes` iterator and stores them in memory, removing them from the vector
    ///
    /// For heap and reference types, the actual value is stored in memory and a pointer is returned
    pub fn load_constant_instructions(
        &self,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
        compilation_ctx: &CompilationContext,
    ) -> Result<(), IntermediateTypeError> {
        match self {
            IntermediateType::IBool => IBool::load_constant_instructions(builder, bytes)?,
            IntermediateType::IU8 => IU8::load_constant_instructions(builder, bytes)?,
            IntermediateType::IU16 => IU16::load_constant_instructions(builder, bytes)?,
            IntermediateType::IU32 => IU32::load_constant_instructions(builder, bytes)?,
            IntermediateType::IU64 => IU64::load_constant_instructions(builder, bytes)?,
            IntermediateType::IU128 => {
                IU128::load_constant_instructions(module, builder, bytes, compilation_ctx)?
            }
            IntermediateType::IU256 => {
                IU256::load_constant_instructions(module, builder, bytes, compilation_ctx)?
            }
            IntermediateType::IAddress => {
                IAddress::load_constant_instructions(module, builder, bytes, compilation_ctx)?
            }
            IntermediateType::ISigner => return Err(IntermediateTypeError::SignerCannotBeConstant),
            IntermediateType::IVector(inner) => {
                IVector::load_constant_instructions(inner, module, builder, bytes, compilation_ctx)?
            }
            IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                return Err(IntermediateTypeError::CannotLoadConstantForReferenceType);
            }
            IntermediateType::IStruct { .. } | IntermediateType::IGenericStructInstance { .. } => {
                return Err(IntermediateTypeError::StructsCannotBeConstants);
            }
            IntermediateType::ITypeParameter(_) => {
                return Err(IntermediateTypeError::FoundTypeParameter);
            }
            IntermediateType::IEnum { .. } | IntermediateType::IGenericEnumInstance { .. } => {
                return Err(IntermediateTypeError::EnumVariantsCannotBeConstants);
            }
        }

        Ok(())
    }

    pub fn move_local_instructions(
        &self,
        builder: &mut InstrSeqBuilder,
        compilation_ctx: &CompilationContext,
        local: LocalId,
    ) -> Result<(), IntermediateTypeError> {
        builder.local_get(local);
        match self {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32 => {
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
            }
            IntermediateType::IU64 => {
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
                        LoadKind::I64 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );
            }
            IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::IAddress
            | IntermediateType::ISigner
            | IntermediateType::IVector(_)
            | IntermediateType::IStruct { .. }
            | IntermediateType::IGenericStructInstance { .. }
            | IntermediateType::IEnum { .. }
            | IntermediateType::IGenericEnumInstance { .. } => {
                builder.load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            }
            IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {}
            IntermediateType::ITypeParameter(_) => {
                return Err(IntermediateTypeError::FoundTypeParameter);
            }
        }

        Ok(())
    }

    pub fn copy_local_instructions(
        &self,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        compilation_ctx: &CompilationContext,
        module_data: &ModuleData,
        local: LocalId,
    ) -> Result<(), IntermediateTypeError> {
        builder.local_get(local);
        match self {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32 => {
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
            }
            IntermediateType::IU64 => {
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
                        LoadKind::I64 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );
            }
            IntermediateType::IU128 => {
                builder.load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
                let copy_f = RuntimeFunction::CopyU128.get(module, Some(compilation_ctx))?;
                builder.call(copy_f);
            }
            IntermediateType::IU256 | IntermediateType::IAddress => {
                builder.load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
                let copy_f = RuntimeFunction::CopyU256.get(module, Some(compilation_ctx))?;
                builder.call(copy_f);
            }
            IntermediateType::IVector(inner_type) => {
                builder.load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
                builder.i32_const(1); // This is the length "multiplier", i.e. length * multiplier = capacity
                IVector::copy_local_instructions(
                    inner_type,
                    module,
                    builder,
                    compilation_ctx,
                    module_data,
                )?;
            }
            IntermediateType::IStruct {
                module_id, index, ..
            } => {
                let struct_ = compilation_ctx.get_struct_by_index(module_id, *index)?;
                builder.load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
                struct_.copy_local_instructions(module, builder, compilation_ctx, module_data)?;
            }
            IntermediateType::IGenericStructInstance {
                module_id,
                index,
                types,
                ..
            } => {
                let struct_ = compilation_ctx.get_struct_by_index(module_id, *index)?;
                let struct_instance = struct_.instantiate(types);
                builder.load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
                struct_instance.copy_local_instructions(
                    module,
                    builder,
                    compilation_ctx,
                    module_data,
                )?;
            }
            IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                // Nothing to be done, pointer is already correct
            }
            IntermediateType::ISigner => {
                return Err(IntermediateTypeError::SignerCannotBeCopied);
            }
            IntermediateType::ITypeParameter(_) => {
                return Err(IntermediateTypeError::FoundTypeParameter);
            }
            IntermediateType::IEnum { .. } | IntermediateType::IGenericEnumInstance { .. } => {
                let enum_ = compilation_ctx.get_enum_by_intermediate_type(self)?;
                builder.load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
                enum_.copy_local_instructions(module, builder, compilation_ctx, module_data)?;
            }
        }

        Ok(())
    }

    pub fn add_load_memory_to_local_instructions(
        &self,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        pointer: LocalId,
        memory: MemoryId,
    ) -> Result<LocalId, IntermediateTypeError> {
        match self {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::IAddress
            | IntermediateType::ISigner
            | IntermediateType::IVector(_)
            | IntermediateType::IRef(_)
            | IntermediateType::IMutRef(_)
            | IntermediateType::IStruct { .. }
            | IntermediateType::IGenericStructInstance { .. }
            | IntermediateType::IEnum { .. }
            | IntermediateType::IGenericEnumInstance { .. } => {
                let local = module.locals.add(ValType::I32);

                builder.local_get(pointer);
                builder.load(
                    memory,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
                builder.local_set(local);

                Ok(local)
            }
            IntermediateType::IU64 => {
                let local = module.locals.add(ValType::I64);

                builder.local_get(pointer);
                builder.load(
                    memory,
                    LoadKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
                builder.local_set(local);

                Ok(local)
            }
            IntermediateType::ITypeParameter(_) => Err(IntermediateTypeError::FoundTypeParameter),
        }
    }

    /// Adds the instructions to load the value into a local variable
    /// Pops the next value from the stack and stores it in the a variable
    pub fn add_stack_to_local_instructions(
        &self,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
    ) -> Result<LocalId, IntermediateTypeError> {
        match self {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::IVector(_)
            | IntermediateType::IAddress
            | IntermediateType::ISigner
            | IntermediateType::IRef(_)
            | IntermediateType::IMutRef(_)
            | IntermediateType::IStruct { .. }
            | IntermediateType::IGenericStructInstance { .. }
            | IntermediateType::IEnum { .. }
            | IntermediateType::IGenericEnumInstance { .. } => {
                let local = module.locals.add(ValType::I32);
                builder.local_set(local);
                Ok(local)
            }
            IntermediateType::IU64 => {
                let local = module.locals.add(ValType::I64);
                builder.local_set(local);
                Ok(local)
            }
            IntermediateType::ITypeParameter(_) => Err(IntermediateTypeError::FoundTypeParameter),
        }
    }

    pub fn add_read_ref_instructions(
        &self,
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
        module_data: &ModuleData,
    ) -> Result<(), IntermediateTypeError> {
        builder.load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );
        match self {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32 => {
                builder.load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            }
            IntermediateType::IU64 => {
                builder.load(
                    compilation_ctx.memory_id,
                    LoadKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            }
            IntermediateType::IU128 => {
                let copy_f = RuntimeFunction::CopyU128.get(module, Some(compilation_ctx))?;
                builder.call(copy_f);
            }
            IntermediateType::IU256 | IntermediateType::IAddress => {
                let copy_f = RuntimeFunction::CopyU256.get(module, Some(compilation_ctx))?;
                builder.call(copy_f);
            }
            IntermediateType::IVector(inner_type) => {
                builder.i32_const(1); // Length multiplier
                IVector::copy_local_instructions(
                    inner_type,
                    module,
                    builder,
                    compilation_ctx,
                    module_data,
                )?;
            }
            IntermediateType::IStruct { .. } | IntermediateType::IGenericStructInstance { .. } => {
                let struct_ = compilation_ctx.get_struct_by_intermediate_type(self)?;
                struct_.copy_local_instructions(module, builder, compilation_ctx, module_data)?;
            }
            IntermediateType::ISigner => {
                // Signer type is read-only, we push the pointer only
            }
            IntermediateType::IEnum { .. } | IntermediateType::IGenericEnumInstance { .. } => {
                let enum_ = compilation_ctx.get_enum_by_intermediate_type(self)?;
                enum_.copy_local_instructions(module, builder, compilation_ctx, module_data)?;
            }
            _ => return Err(IntermediateTypeError::CannotReadRefOfType(self.clone())),
        }

        Ok(())
    }

    pub fn add_write_ref_instructions(
        &self,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        compilation_ctx: &CompilationContext,
    ) -> Result<(), IntermediateTypeError> {
        match self {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64 => {
                let (val_type, store_kind) = if *self == IntermediateType::IU64 {
                    (ValType::I64, StoreKind::I64 { atomic: false })
                } else {
                    (ValType::I32, StoreKind::I32 { atomic: false })
                };
                let val = module.locals.add(val_type);
                let ptr = module.locals.add(ValType::I32);
                builder
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_set(ptr)
                    .local_set(val)
                    .local_get(ptr)
                    .local_get(val)
                    .store(
                        compilation_ctx.memory_id,
                        store_kind,
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );
            }
            IntermediateType::IU128 | IntermediateType::IU256 | IntermediateType::IAddress => {
                let src_ptr = module.locals.add(ValType::I32); // what to copy
                let ref_ptr = module.locals.add(ValType::I32); // where to copy

                // Pop the reference and value pointers from the stack
                builder
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_set(ref_ptr)
                    .local_set(src_ptr);

                let bytes = match self {
                    IntermediateType::IU128 => 16,
                    _ => 32,
                };

                // Copy memory in 8-byte chunks
                for offset in (0..bytes).step_by(8) {
                    // destination address
                    builder
                        .local_get(ref_ptr)
                        .i32_const(offset)
                        .binop(BinaryOp::I32Add);

                    // source address
                    builder
                        .local_get(src_ptr)
                        .i32_const(offset)
                        .binop(BinaryOp::I32Add)
                        .load(
                            compilation_ctx.memory_id,
                            LoadKind::I64 { atomic: false },
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        );

                    // store at destination address
                    builder.store(
                        compilation_ctx.memory_id,
                        StoreKind::I64 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );
                }
            }
            // We just update the intermediate pointer, since the new values are already allocated
            // in memory
            IntermediateType::IVector(_)
            | IntermediateType::IStruct { .. }
            | IntermediateType::IGenericStructInstance { .. }
            | IntermediateType::IEnum { .. }
            | IntermediateType::IGenericEnumInstance { .. } => {
                // Since the memory needed for vectors might differ, we don't overwrite it.
                // We update the inner pointer to point to the location where the new vector is already allocated.
                let src_ptr = module.locals.add(ValType::I32);
                let ref_ptr = module.locals.add(ValType::I32);

                // Swap pointers order in the stack
                builder.swap(ref_ptr, src_ptr);

                // Store src_ptr at ref_ptr
                // Now the inner pointer is updated to point to the new vector/struct
                builder.store(
                    compilation_ctx.memory_id,
                    StoreKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            }
            IntermediateType::ISigner => {
                return Err(IntermediateTypeError::CannotWriteRefOnSigner);
            }
            IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                return Err(IntermediateTypeError::FoundReferenceOfReference);
            }
            IntermediateType::ITypeParameter(_) => {
                return Err(IntermediateTypeError::FoundTypeParameter);
            }
        }

        Ok(())
    }

    pub fn box_local_instructions(
        &self,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        compilation_ctx: &CompilationContext,
        local: LocalId,
    ) -> Result<(), IntermediateTypeError> {
        match self {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32 => {
                let val = module.locals.add(ValType::I32);
                let ptr = module.locals.add(ValType::I32);
                builder
                    .local_set(val)
                    .i32_const(4)
                    .call(compilation_ctx.allocator)
                    .local_tee(local)
                    .i32_const(self.stack_data_size()? as i32)
                    .call(compilation_ctx.allocator)
                    .local_tee(ptr)
                    .store(
                        compilation_ctx.memory_id,
                        StoreKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_get(ptr)
                    .local_get(val)
                    .store(
                        compilation_ctx.memory_id,
                        StoreKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );
            }
            IntermediateType::IU64 => {
                let val = module.locals.add(ValType::I64);
                let ptr = module.locals.add(ValType::I32);
                builder
                    .local_set(val)
                    .i32_const(4)
                    .call(compilation_ctx.allocator)
                    .local_tee(local)
                    .i32_const(8)
                    .call(compilation_ctx.allocator)
                    .local_tee(ptr)
                    .store(
                        compilation_ctx.memory_id,
                        StoreKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_get(ptr)
                    .local_get(val)
                    .store(
                        compilation_ctx.memory_id,
                        StoreKind::I64 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );
            }
            IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::IAddress
            | IntermediateType::ISigner
            | IntermediateType::IVector(_)
            | IntermediateType::IRef(_)
            | IntermediateType::IMutRef(_)
            | IntermediateType::IStruct { .. }
            | IntermediateType::IGenericStructInstance { .. }
            | IntermediateType::IEnum { .. }
            | IntermediateType::IGenericEnumInstance { .. } => {
                let ptr = module.locals.add(ValType::I32);
                builder
                    .local_set(ptr)
                    .i32_const(4)
                    .call(compilation_ctx.allocator)
                    .local_tee(local)
                    .local_get(ptr)
                    .store(
                        compilation_ctx.memory_id,
                        StoreKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );
            }
            IntermediateType::ITypeParameter(_) => {
                return Err(IntermediateTypeError::FoundTypeParameter);
            }
        }

        Ok(())
    }

    pub fn load_equality_instructions(
        &self,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        compilation_ctx: &CompilationContext,
        module_data: &ModuleData,
    ) -> Result<(), IntermediateTypeError> {
        match self {
            Self::IBool | Self::IU8 | Self::IU16 | Self::IU32 => {
                builder.binop(BinaryOp::I32Eq);
            }
            Self::IU64 => {
                builder.binop(BinaryOp::I64Eq);
            }
            Self::IU128 => IU128::equality(builder, module, compilation_ctx)?,
            Self::IU256 => IU256::equality(builder, module, compilation_ctx)?,
            Self::IAddress => IAddress::equality(builder, module, compilation_ctx)?,
            Self::ISigner => {
                // Signers can only be created by the VM and injected into the smart contract.
                // There can only be one signer, so if we find a situation where signers are
                // compared, we are comparing the same thing.
                builder.i32_const(1);
            }
            Self::IVector(inner) => {
                IVector::equality(builder, module, compilation_ctx, module_data, inner)?
            }
            Self::IStruct {
                index, module_id, ..
            } => {
                let struct_ = compilation_ctx.get_struct_by_index(module_id, *index)?;
                struct_.equality(builder, module, compilation_ctx, module_data)?
            }
            Self::IGenericStructInstance {
                index,
                module_id,
                types,
                ..
            } => {
                let struct_ = compilation_ctx.get_struct_by_index(module_id, *index)?;
                struct_.instantiate(types).equality(
                    builder,
                    module,
                    compilation_ctx,
                    module_data,
                )?
            }
            Self::IEnum { .. } | Self::IGenericEnumInstance { .. } => {
                let enum_ = compilation_ctx.get_enum_by_intermediate_type(self)?;
                enum_.equality(builder, module, compilation_ctx, module_data)?;
            }
            Self::IRef(inner) | Self::IMutRef(inner) => {
                let ptr1 = module.locals.add(ValType::I32);
                let ptr2 = module.locals.add(ValType::I32);

                // Load the intermediate pointers
                builder
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_set(ptr1)
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_set(ptr2);

                match inner.as_ref() {
                    // If inner is a simple type, we load te value into stack
                    IntermediateType::IBool
                    | IntermediateType::IU8
                    | IntermediateType::IU16
                    | IntermediateType::IU32
                    | IntermediateType::IU64 => {
                        builder
                            .local_get(ptr1)
                            .load(
                                compilation_ctx.memory_id,
                                if **inner == IntermediateType::IU64 {
                                    LoadKind::I64 { atomic: false }
                                } else {
                                    LoadKind::I32 { atomic: false }
                                },
                                MemArg {
                                    align: 0,
                                    offset: 0,
                                },
                            )
                            .local_get(ptr2)
                            .load(
                                compilation_ctx.memory_id,
                                if **inner == IntermediateType::IU64 {
                                    LoadKind::I64 { atomic: false }
                                } else {
                                    LoadKind::I32 { atomic: false }
                                },
                                MemArg {
                                    align: 0,
                                    offset: 0,
                                },
                            );
                    }
                    // If inner is a heap type, we already loaded the value of intermediate
                    // pointers, so we load them
                    IntermediateType::IU128
                    | IntermediateType::IU256
                    | IntermediateType::IAddress
                    | IntermediateType::ISigner
                    | IntermediateType::IVector(_)
                    | IntermediateType::IStruct { .. }
                    | IntermediateType::IGenericStructInstance { .. }
                    | IntermediateType::IEnum { .. }
                    | IntermediateType::IGenericEnumInstance { .. } => {
                        builder.local_get(ptr1).local_get(ptr2);
                    }
                    IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                        return Err(IntermediateTypeError::FoundReferenceOfReference);
                    }
                    IntermediateType::ITypeParameter(_) => {
                        return Err(IntermediateTypeError::FoundTypeParameter);
                    }
                }

                inner.load_equality_instructions(module, builder, compilation_ctx, module_data)?
            }

            IntermediateType::ITypeParameter(_) => {
                return Err(IntermediateTypeError::FoundTypeParameter);
            }
        }

        Ok(())
    }

    pub fn load_not_equality_instructions(
        &self,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        compilation_ctx: &CompilationContext,
        module_data: &ModuleData,
    ) -> Result<(), TranslationError> {
        self.load_equality_instructions(module, builder, compilation_ctx, module_data)?;
        builder.negate();
        Ok(())
    }

    /// Returns true if the type is a stack type (the value is directly hanndled in wasm stack
    /// instead of handling a pointer), otherwise returns false.
    pub fn is_stack_type(&self) -> Result<bool, IntermediateTypeError> {
        match self {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64 => Ok(true),
            IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::IAddress
            | IntermediateType::ISigner
            | IntermediateType::IVector(_)
            | IntermediateType::IRef(_)
            | IntermediateType::IMutRef(_)
            | IntermediateType::IStruct { .. }
            | IntermediateType::IGenericStructInstance { .. }
            | IntermediateType::IEnum { .. }
            | IntermediateType::IGenericEnumInstance { .. } => Ok(false),
            IntermediateType::ITypeParameter(_) => Err(IntermediateTypeError::FoundTypeParameter),
        }
    }

    pub fn get_name(
        &self,
        compilation_ctx: &CompilationContext,
    ) -> Result<String, IntermediateTypeError> {
        let name = match self {
            IntermediateType::IBool => "bool".to_string(),
            IntermediateType::IU8 => "u8".to_string(),
            IntermediateType::IU16 => "u16".to_string(),
            IntermediateType::IU32 => "u32".to_string(),
            IntermediateType::IU64 => "u64".to_string(),
            IntermediateType::IU128 => "u128".to_string(),
            IntermediateType::IU256 => "u256".to_string(),
            IntermediateType::IAddress => "address".to_string(),
            IntermediateType::ISigner => "signer".to_string(),
            IntermediateType::IVector(inner) => {
                format!("vector<{}>", inner.get_name(compilation_ctx)?)
            }
            IntermediateType::IRef(inner) => format!("&{}", inner.get_name(compilation_ctx)?),
            IntermediateType::IMutRef(inner) => {
                format!("&mut {}", inner.get_name(compilation_ctx)?)
            }
            IntermediateType::IStruct {
                module_id, index, ..
            } => {
                let struct_ = compilation_ctx.get_struct_by_index(module_id, *index)?;

                struct_.identifier.clone()
            }
            IntermediateType::IGenericStructInstance {
                module_id,
                index,
                types,
                ..
            } => {
                let struct_ = compilation_ctx.get_struct_by_index(module_id, *index)?;

                let types = types
                    .iter()
                    .map(|t| t.get_name(compilation_ctx))
                    .collect::<Result<Vec<String>, IntermediateTypeError>>()?
                    .join(",");

                format!("{}<{types}>", struct_.identifier.clone())
            }
            IntermediateType::ITypeParameter(_) => {
                return Err(IntermediateTypeError::FoundTypeParameter);
            }
            IntermediateType::IEnum { .. } | IntermediateType::IGenericEnumInstance { .. } => {
                let enum_ = compilation_ctx.get_enum_by_intermediate_type(self)?;
                enum_.identifier.clone()
            }
        };

        Ok(name)
    }

    /// Returns the hash of the type
    pub fn get_hash(
        &self,
        compilation_ctx: &CompilationContext,
    ) -> Result<u64, IntermediateTypeError> {
        let mut hasher = get_hasher();
        self.process_hash(&mut hasher, compilation_ctx)?;
        Ok(hasher.finish())
    }

    pub fn process_hash(
        &self,
        mut hasher: &mut dyn Hasher,
        compilation_ctx: &CompilationContext,
    ) -> Result<(), IntermediateTypeError> {
        match self {
            IntermediateType::IStruct {
                module_id, index, ..
            } => {
                let struct_ = compilation_ctx.get_struct_by_index(module_id, *index)?;
                let module_data = compilation_ctx.get_module_data_by_id(module_id)?;
                if let Some(external_struct) = module_data
                    .special_attributes
                    .external_struct
                    .get(struct_.identifier.as_str())
                {
                    let foreign_module_id = ModuleId {
                        address: Address::from_bytes(external_struct.address),
                        module_name: external_struct.module_name.clone(),
                    };

                    Hash::hash(&foreign_module_id, &mut hasher);
                } else {
                    Hash::hash(&module_id, &mut hasher);
                }
                struct_.identifier.hash(&mut hasher);
            }
            IntermediateType::IGenericStructInstance {
                module_id,
                index,
                types,
                ..
            } => {
                let struct_ = compilation_ctx.get_struct_by_index(module_id, *index)?;
                let module_data = compilation_ctx.get_module_data_by_id(module_id)?;
                if let Some(external_struct) = module_data
                    .special_attributes
                    .external_struct
                    .get(struct_.identifier.as_str())
                {
                    let foreign_module_id = ModuleId {
                        address: Address::from_bytes(external_struct.address),
                        module_name: external_struct.module_name.clone(),
                    };

                    Hash::hash(&foreign_module_id, &mut hasher);
                } else {
                    Hash::hash(&module_id, &mut hasher);
                }

                for t in types {
                    t.process_hash(&mut hasher, compilation_ctx)?;
                }
                struct_.identifier.hash(&mut hasher);
            }
            _ => {
                self.hash(&mut hasher);
            }
        }

        Ok(())
    }
}

impl TryFrom<&IntermediateType> for ValType {
    type Error = IntermediateTypeError;

    fn try_from(value: &IntermediateType) -> Result<Self, Self::Error> {
        match value {
            IntermediateType::IU64 => Ok(ValType::I64),
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::IAddress
            | IntermediateType::ISigner
            | IntermediateType::IVector(_)
            | IntermediateType::IRef(_)
            | IntermediateType::IMutRef(_)
            | IntermediateType::IStruct { .. }
            | IntermediateType::IGenericStructInstance { .. }
            | IntermediateType::IEnum { .. }
            | IntermediateType::IGenericEnumInstance { .. } => Ok(ValType::I32),
            IntermediateType::ITypeParameter(_) => Err(IntermediateTypeError::FoundTypeParameter),
        }
    }
}

impl TryFrom<IntermediateType> for ValType {
    type Error = IntermediateTypeError;

    fn try_from(value: IntermediateType) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

#[derive(Debug)]
pub struct ISignature {
    pub arguments: Vec<IntermediateType>,
    pub returns: Vec<IntermediateType>,
}

impl ISignature {
    pub fn from_signatures(
        arguments: &Signature,
        returns: &Signature,
        handles_map: &HashMap<DatatypeHandleIndex, UserDefinedType>,
    ) -> Result<Self, IntermediateTypeError> {
        let arguments = arguments
            .0
            .iter()
            .map(|token| IntermediateType::try_from_signature_token(token, handles_map))
            .collect::<Result<Vec<IntermediateType>, IntermediateTypeError>>()?;

        let returns = returns
            .0
            .iter()
            .map(|token| IntermediateType::try_from_signature_token(token, handles_map))
            .collect::<Result<Vec<IntermediateType>, IntermediateTypeError>>()?;

        Ok(Self { arguments, returns })
    }

    /// Returns the wasm types of the return values
    ///
    /// If the function has return values, the return type will always be a tuple (represented by an I32 pointer),
    /// as the multi-value return feature is not enabled in Stylus VM.
    pub fn get_return_wasm_types(&self) -> Vec<ValType> {
        if self.returns.is_empty() {
            vec![]
        } else {
            vec![ValType::I32]
        }
    }

    pub fn get_argument_wasm_types(&self) -> Result<Vec<ValType>, IntermediateTypeError> {
        self.arguments.iter().map(ValType::try_from).collect()
    }
}
