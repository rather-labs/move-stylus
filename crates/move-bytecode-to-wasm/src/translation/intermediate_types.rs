pub mod address;
pub mod boolean;
pub mod enums;
pub mod error;
pub mod heap_integers;
pub mod signer;
pub mod simple_integers;
pub mod structs;
pub(crate) mod user_type_fields;
pub mod vector;

use std::{
    borrow::Cow,
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::Arc,
};

use crate::{
    CompilationContext, UserDefinedType,
    compilation_context::{ModuleId, module_data::Address},
    data::RuntimeErrorData,
    hasher::get_hasher,
    runtime::RuntimeFunction,
    wasm_builder_extensions::WasmBuilderExtension,
};

use address::IAddress;
use boolean::IBool;
use error::IntermediateTypeError;
use heap_integers::{IU128, IU256};
use move_binary_format::file_format::{DatatypeHandleIndex, Signature, SignatureToken};
use move_symbol_pool::Symbol;
use simple_integers::{IU8, IU16, IU32, IU64};
use vector::IVector;

use walrus::{
    InstrSeqBuilder, LocalId, MemoryId, Module, ValType,
    ir::{BinaryOp, ExtendedLoad, LoadKind, MemArg, StoreKind},
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
    IVector(Arc<IntermediateType>),
    IRef(Arc<IntermediateType>),
    IMutRef(Arc<IntermediateType>),

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
    pub fn wasm_memory_data_size(&self) -> Result<i32, IntermediateTypeError> {
        let size = match self {
            IntermediateType::IBool | IntermediateType::IU8 => 1,
            IntermediateType::IU16 => 2,
            IntermediateType::IU32
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
            IntermediateType::IU64 => 8,
            IntermediateType::ITypeParameter(_) => {
                return Err(IntermediateTypeError::FoundTypeParameter);
            }
        };

        Ok(size)
    }

    pub fn store_kind(&self) -> Result<StoreKind, IntermediateTypeError> {
        let store_kind = match self {
            IntermediateType::IBool | IntermediateType::IU8 => StoreKind::I32_8 { atomic: false },
            IntermediateType::IU16 => StoreKind::I32_16 { atomic: false },
            IntermediateType::IU32
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
            | IntermediateType::IGenericEnumInstance { .. } => StoreKind::I32 { atomic: false },
            IntermediateType::IU64 => StoreKind::I64 { atomic: false },
            IntermediateType::ITypeParameter(_) => {
                return Err(IntermediateTypeError::FoundTypeParameter);
            }
        };
        Ok(store_kind)
    }

    pub fn load_kind(&self) -> Result<LoadKind, IntermediateTypeError> {
        let load_kind = match self {
            IntermediateType::IBool | IntermediateType::IU8 => LoadKind::I32_8 {
                kind: ExtendedLoad::ZeroExtend,
            },
            IntermediateType::IU16 => LoadKind::I32_16 {
                kind: ExtendedLoad::ZeroExtend,
            },
            IntermediateType::IU32
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
            | IntermediateType::IGenericEnumInstance { .. } => LoadKind::I32 { atomic: false },
            IntermediateType::IU64 => LoadKind::I64 { atomic: false },
            IntermediateType::ITypeParameter(_) => {
                return Err(IntermediateTypeError::FoundTypeParameter);
            }
        };
        Ok(load_kind)
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
                Ok(IntermediateType::IVector(Arc::new(itoken)))
            }
            SignatureToken::Reference(token) => {
                let itoken = Self::try_from_signature_token(token.as_ref(), handles_map)?;
                Ok(IntermediateType::IRef(Arc::new(itoken)))
            }
            SignatureToken::MutableReference(token) => {
                let itoken = Self::try_from_signature_token(token.as_ref(), handles_map)?;
                Ok(IntermediateType::IMutRef(Arc::new(itoken)))
            }
            SignatureToken::Datatype(datatype_index) => {
                if let Some(udt) = handles_map.get(datatype_index) {
                    Ok(match udt {
                        UserDefinedType::Struct { module_id, index } => IntermediateType::IStruct {
                            module_id: *module_id,
                            index: *index,
                            vm_handled_struct: VmHandledStruct::None,
                        },
                        UserDefinedType::Enum { module_id, index } => IntermediateType::IEnum {
                            module_id: *module_id,
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
                                module_id: *module_id,
                                index: *index,
                                types,
                                vm_handled_struct: VmHandledStruct::None,
                            }
                        }
                        UserDefinedType::Enum { module_id, index } => {
                            IntermediateType::IGenericEnumInstance {
                                module_id: *module_id,
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
        bytes: &mut std::slice::Iter<'_, u8>,
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
            IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {}
            IntermediateType::ITypeParameter(_) => {
                return Err(IntermediateTypeError::FoundTypeParameter);
            }
            _ => {
                // Load the middle ptr
                builder.load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

                // For stach types, load the value
                match self {
                    IntermediateType::IBool
                    | IntermediateType::IU8
                    | IntermediateType::IU16
                    | IntermediateType::IU32
                    | IntermediateType::IU64 => {
                        builder.load(
                            compilation_ctx.memory_id,
                            self.load_kind()?,
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        );
                    }
                    _ => {
                        // For heap types we can't load the value directly, so the ptr is ok
                    }
                }
            }
        }

        Ok(())
    }

    pub fn copy_local_instructions(
        &self,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        compilation_ctx: &CompilationContext,
        runtime_error_data: &mut RuntimeErrorData,
        local: LocalId,
    ) -> Result<(), IntermediateTypeError> {
        builder.local_get(local);
        match self {
            IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                // Forward the local directly
            }
            IntermediateType::ISigner => {
                return Err(IntermediateTypeError::SignerCannotBeCopied);
            }
            IntermediateType::ITypeParameter(_) => {
                return Err(IntermediateTypeError::FoundTypeParameter);
            }
            _ => {
                // Load the middle ptr
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
                    | IntermediateType::IU32
                    | IntermediateType::IU64 => {
                        builder.load(
                            compilation_ctx.memory_id,
                            self.load_kind()?,
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        );
                    }
                    IntermediateType::IU128 => {
                        let copy_f =
                            RuntimeFunction::CopyU128.get(module, Some(compilation_ctx), None)?;
                        builder.call(copy_f);
                    }
                    IntermediateType::IU256 | IntermediateType::IAddress => {
                        let copy_f =
                            RuntimeFunction::CopyU256.get(module, Some(compilation_ctx), None)?;
                        builder.call(copy_f);
                    }
                    IntermediateType::IVector(inner_type) => {
                        builder.i32_const(1); // This is the length "multiplier", i.e. length * multiplier = capacity
                        let copy_local_function = RuntimeFunction::VecCopyLocal.get_generic(
                            module,
                            compilation_ctx,
                            Some(runtime_error_data),
                            &[inner_type],
                        )?;
                        builder.call(copy_local_function);
                    }
                    IntermediateType::IStruct { .. }
                    | IntermediateType::IGenericStructInstance { .. } => {
                        let struct_ = compilation_ctx.get_struct_by_intermediate_type(self)?;
                        struct_.copy_local_instructions(
                            module,
                            builder,
                            compilation_ctx,
                            runtime_error_data,
                        )?;
                    }
                    IntermediateType::IEnum { .. }
                    | IntermediateType::IGenericEnumInstance { .. } => {
                        let enum_ = compilation_ctx.get_enum_by_intermediate_type(self)?;
                        enum_.copy_local_instructions(
                            module,
                            builder,
                            compilation_ctx,
                            runtime_error_data,
                        )?;
                    }
                    IntermediateType::ISigner
                    | IntermediateType::IRef(_)
                    | IntermediateType::IMutRef(_)
                    | IntermediateType::ITypeParameter(_) => {
                        // These cases are handled in the outer match.
                        builder.unreachable();
                    }
                }
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
        let val_type = ValType::try_from(self)?;
        let local = module.locals.add(val_type);
        builder
            .local_get(pointer)
            .load(
                memory,
                self.load_kind()?,
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )
            .local_set(local);
        Ok(local)
    }

    /// Adds the instructions to load the value into a local variable
    /// Pops the next value from the stack and stores it in the a variable
    pub fn add_stack_to_local_instructions(
        &self,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
    ) -> Result<LocalId, IntermediateTypeError> {
        let val_type = ValType::try_from(self)?;
        let local = module.locals.add(val_type);
        builder.local_set(local);
        Ok(local)
    }

    pub fn add_read_ref_instructions(
        &self,
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
        runtime_error_data: &mut RuntimeErrorData,
    ) -> Result<(), IntermediateTypeError> {
        // Load the middle ptr
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
            | IntermediateType::IU32
            | IntermediateType::IU64 => {
                builder.load(
                    compilation_ctx.memory_id,
                    self.load_kind()?,
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            }
            IntermediateType::IU128 => {
                let copy_f = RuntimeFunction::CopyU128.get(module, Some(compilation_ctx), None)?;
                builder.call(copy_f);
            }
            IntermediateType::IU256 | IntermediateType::IAddress => {
                let copy_f = RuntimeFunction::CopyU256.get(module, Some(compilation_ctx), None)?;
                builder.call(copy_f);
            }
            IntermediateType::IVector(inner_type) => {
                builder.i32_const(1); // Length multiplier
                let copy_local_function = RuntimeFunction::VecCopyLocal.get_generic(
                    module,
                    compilation_ctx,
                    Some(runtime_error_data),
                    &[inner_type],
                )?;
                builder.call(copy_local_function);
            }
            IntermediateType::IStruct { .. } | IntermediateType::IGenericStructInstance { .. } => {
                let struct_ = compilation_ctx.get_struct_by_intermediate_type(self)?;
                struct_.copy_local_instructions(
                    module,
                    builder,
                    compilation_ctx,
                    runtime_error_data,
                )?;
            }
            IntermediateType::IEnum { .. } | IntermediateType::IGenericEnumInstance { .. } => {
                let enum_ = compilation_ctx.get_enum_by_intermediate_type(self)?;
                enum_.copy_local_instructions(
                    module,
                    builder,
                    compilation_ctx,
                    runtime_error_data,
                )?;
            }
            IntermediateType::ISigner => {
                // Signer type is read-only, we push the pointer only
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
                let val = module.locals.add(ValType::try_from(self)?);
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
                    .swap(ptr, val)
                    .store(
                        compilation_ctx.memory_id,
                        self.store_kind()?,
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
            | IntermediateType::IU32
            | IntermediateType::IU64 => {
                let val = module.locals.add(ValType::try_from(self)?);
                let ptr = module.locals.add(ValType::I32);
                builder.local_set(val);
                builder
                    .i32_const(4)
                    .call(compilation_ctx.allocator)
                    .local_tee(local);

                builder
                    .i32_const(self.wasm_memory_data_size()?)
                    .call(compilation_ctx.allocator)
                    .local_tee(ptr);

                builder
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
                        self.store_kind()?,
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
                builder.local_set(ptr);

                builder
                    .i32_const(4)
                    .call(compilation_ctx.allocator)
                    .local_tee(local);
                builder.local_get(ptr).store(
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
        runtime_error_data: &mut RuntimeErrorData,
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
                let equality_f = RuntimeFunction::VecEquality.get_generic(
                    module,
                    compilation_ctx,
                    Some(runtime_error_data),
                    &[inner],
                )?;
                builder.call(equality_f);
            }
            Self::IStruct {
                index, module_id, ..
            } => {
                let struct_ = compilation_ctx.get_struct_by_index(module_id, *index)?;
                struct_.equality(builder, module, compilation_ctx, runtime_error_data)?
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
                    runtime_error_data,
                )?
            }
            Self::IEnum { .. } | Self::IGenericEnumInstance { .. } => {
                let enum_ = compilation_ctx.get_enum_by_intermediate_type(self)?;
                enum_.equality(builder, module, compilation_ctx, runtime_error_data)?;
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
                                inner.load_kind()?,
                                MemArg {
                                    align: 0,
                                    offset: 0,
                                },
                            )
                            .local_get(ptr2)
                            .load(
                                compilation_ctx.memory_id,
                                inner.load_kind()?,
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

                inner.load_equality_instructions(
                    module,
                    builder,
                    compilation_ctx,
                    runtime_error_data,
                )?
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
        runtime_error_data: &mut RuntimeErrorData,
    ) -> Result<(), TranslationError> {
        self.load_equality_instructions(module, builder, compilation_ctx, runtime_error_data)?;
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

    pub fn get_name<'a>(
        &'a self,
        compilation_ctx: &'a CompilationContext,
    ) -> Result<Cow<'a, str>, IntermediateTypeError> {
        let name = match self {
            IntermediateType::IBool => Cow::Borrowed("bool"),
            IntermediateType::IU8 => Cow::Borrowed("u8"),
            IntermediateType::IU16 => Cow::Borrowed("u16"),
            IntermediateType::IU32 => Cow::Borrowed("u32"),
            IntermediateType::IU64 => Cow::Borrowed("u64"),
            IntermediateType::IU128 => Cow::Borrowed("u128"),
            IntermediateType::IU256 => Cow::Borrowed("u256"),
            IntermediateType::IAddress => Cow::Borrowed("address"),
            IntermediateType::ISigner => Cow::Borrowed("signer"),
            IntermediateType::IVector(inner) => {
                Cow::Owned(format!("vector<{}>", inner.get_name(compilation_ctx)?))
            }
            IntermediateType::IRef(inner) => {
                Cow::Owned(format!("&{}", inner.get_name(compilation_ctx)?))
            }
            IntermediateType::IMutRef(inner) => {
                Cow::Owned(format!("&mut {}", inner.get_name(compilation_ctx)?))
            }
            IntermediateType::IStruct {
                module_id, index, ..
            } => {
                let struct_ = compilation_ctx.get_struct_by_index(module_id, *index)?;

                Cow::Borrowed(struct_.identifier.as_str())
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
                    .collect::<Result<Vec<Cow<str>>, IntermediateTypeError>>()?
                    .join(",");

                Cow::Owned(format!("{}<{types}>", struct_.identifier.clone()))
            }
            IntermediateType::ITypeParameter(_) => {
                return Err(IntermediateTypeError::FoundTypeParameter);
            }
            IntermediateType::IEnum { index, module_id } => {
                let enum_ = compilation_ctx.get_enum_by_index(module_id, *index)?;
                Cow::Borrowed(enum_.identifier.as_str())
            }
            IntermediateType::IGenericEnumInstance {
                module_id,
                index,
                types,
                ..
            } => {
                let struct_ = compilation_ctx.get_enum_by_index(module_id, *index)?;

                let types = types
                    .iter()
                    .map(|t| t.get_name(compilation_ctx))
                    .collect::<Result<Vec<Cow<str>>, IntermediateTypeError>>()?
                    .join(",");

                Cow::Owned(format!("{}<{types}>", struct_.identifier.clone()))
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
                    .get(&struct_.identifier)
                {
                    let foreign_module_id = ModuleId::new(
                        Address::from_bytes(external_struct.address),
                        external_struct.module_name.as_str(),
                    );

                    Hash::hash(&foreign_module_id, &mut hasher);
                } else {
                    Hash::hash(&module_id, &mut hasher);
                }
                struct_.identifier.as_str().hash(&mut hasher);
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
                    .get(&struct_.identifier)
                {
                    let foreign_module_id = ModuleId {
                        address: Address::from_bytes(external_struct.address),
                        module_name: Symbol::from(external_struct.module_name.as_str()),
                    };

                    Hash::hash(&foreign_module_id, &mut hasher);
                } else {
                    Hash::hash(&module_id, &mut hasher);
                }

                for t in types {
                    t.process_hash(&mut hasher, compilation_ctx)?;
                }
                struct_.identifier.as_str().hash(&mut hasher);
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
