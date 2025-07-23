use std::collections::HashMap;

use crate::{
    CompilationContext, UserDefinedType,
    compilation_context::{ModuleData, ModuleId},
    runtime::RuntimeFunction,
    wasm_builder_extensions::WasmBuilderExtension,
};
use address::IAddress;
use boolean::IBool;
use heap_integers::{IU128, IU256};
use move_binary_format::file_format::{DatatypeHandleIndex, Signature, SignatureToken};
use simple_integers::{IU8, IU16, IU32, IU64};
use structs::IStruct;
use vector::IVector;
use walrus::{
    InstrSeqBuilder, LocalId, MemoryId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

pub mod address;
pub mod boolean;
pub mod enums;
pub mod heap_integers;
pub mod reference;
pub mod signer;
pub mod simple_integers;
pub mod structs;
pub mod vector;

#[derive(Clone, PartialEq, Debug, Eq)]
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
    IStruct(u16),

    /// The usize is the index of the generic struct.
    /// The Vec<IntermediateType> is the list of types we are going to instantiate the generic
    /// struct with.
    IGenericStructInstance(u16, Vec<IntermediateType>),

    /// Intermediate enum representation
    ///
    /// The first u16 is the enum's index in the compilation context.
    IEnum(u16),

    /// Represents a complex datatype (struct or enum) defined in another module.
    ///
    /// When recursively processing the dependencies, we don't have the complete information about
    /// the type (whether is a struct or an enum, its fields, etc), so we save a refrence to it. At
    /// the moment of processing it, we should have all the dependencies processed with its
    /// corresponding data.
    IExternalUserData {
        module_id: ModuleId,
        identifier: String,
    },
}

impl IntermediateType {
    /// Returns the size in bytes, that this type needs in memory to be stored
    pub fn stack_data_size(&self) -> u32 {
        match self {
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
            | IntermediateType::IStruct(_)
            | IntermediateType::IEnum(_)
            | IntermediateType::IGenericStructInstance(_, _)
            | IntermediateType::IExternalUserData { .. } => 4,
            IntermediateType::ITypeParameter(_) => {
                panic!("type parameter does not have a known stack data size at compile time")
            }
        }
    }

    pub fn try_from_signature_token(
        value: &SignatureToken,
        handles_map: &HashMap<DatatypeHandleIndex, UserDefinedType>,
    ) -> Result<Self, anyhow::Error> {
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
            SignatureToken::Datatype(index) => {
                if let Some(udt) = handles_map.get(index) {
                    Ok(match udt {
                        UserDefinedType::Struct(i) => IntermediateType::IStruct(*i),
                        UserDefinedType::Enum(i) => IntermediateType::IEnum(*i as u16),
                        UserDefinedType::ExternalData {
                            module: module_id,
                            identifier,
                        } => IntermediateType::IExternalUserData {
                            module_id: module_id.clone(),
                            identifier: identifier.clone(),
                        },
                    })
                } else {
                    Err(anyhow::anyhow!(
                        "No user defined data with handler index: {index:?} found"
                    ))
                }
            }
            SignatureToken::DatatypeInstantiation(index) => {
                if let Some(udt) = handles_map.get(&index.0) {
                    let types = index
                        .1
                        .iter()
                        .map(|t| Self::try_from_signature_token(t, handles_map))
                        .collect::<Result<Vec<IntermediateType>, anyhow::Error>>()?;

                    Ok(match udt {
                        UserDefinedType::Struct(i) => {
                            IntermediateType::IGenericStructInstance(*i, types)
                        }
                        UserDefinedType::Enum(_) => todo!(),
                        UserDefinedType::ExternalData {
                            module: module_id,
                            identifier,
                        } => IntermediateType::IExternalUserData {
                            module_id: module_id.clone(),
                            identifier: identifier.clone(),
                        },
                    })
                } else {
                    Err(anyhow::anyhow!(
                        "No user defined data with handler index: {index:?} found"
                    ))
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
    ) {
        match self {
            IntermediateType::IBool => IBool::load_constant_instructions(builder, bytes),
            IntermediateType::IU8 => IU8::load_constant_instructions(builder, bytes),
            IntermediateType::IU16 => IU16::load_constant_instructions(builder, bytes),
            IntermediateType::IU32 => IU32::load_constant_instructions(builder, bytes),
            IntermediateType::IU64 => IU64::load_constant_instructions(builder, bytes),
            IntermediateType::IU128 => {
                IU128::load_constant_instructions(module, builder, bytes, compilation_ctx)
            }
            IntermediateType::IU256 => {
                IU256::load_constant_instructions(module, builder, bytes, compilation_ctx)
            }
            IntermediateType::IAddress => {
                IAddress::load_constant_instructions(module, builder, bytes, compilation_ctx)
            }
            IntermediateType::ISigner => panic!("signer type can't be loaded as a constant"),
            IntermediateType::IVector(inner) => {
                IVector::load_constant_instructions(inner, module, builder, bytes, compilation_ctx)
            }
            IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                panic!("cannot load a constant for a reference type");
            }
            IntermediateType::IStruct(_) | IntermediateType::IGenericStructInstance(_, _) => {
                panic!("structs can't be loaded as constants")
            }
            IntermediateType::IExternalUserData { .. } => {
                panic!("external user data can't be loaded as constants")
            }
            IntermediateType::ITypeParameter(_) => {
                panic!("can't load a type parameter as a constant, expected a concrete type");
            }
            IntermediateType::IEnum(_) => todo!(),
        }
    }

    pub fn move_local_instructions(
        &self,
        builder: &mut InstrSeqBuilder,
        compilation_ctx: &CompilationContext,
        local: LocalId,
    ) {
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
            | IntermediateType::IStruct(_)
            | IntermediateType::IGenericStructInstance(_, _)
            | IntermediateType::IEnum(_)
            | IntermediateType::IExternalUserData { .. } => {
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
                panic!("cannot move a type parameter, expected a concrete type");
            }
        }
    }

    pub fn copy_local_instructions(
        &self,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        compilation_ctx: &CompilationContext,
        module_data: &ModuleData,
        local: LocalId,
    ) {
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
                let copy_f = RuntimeFunction::CopyU128.get(module, Some(compilation_ctx));
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
                let copy_f = RuntimeFunction::CopyU256.get(module, Some(compilation_ctx));
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
                );
            }
            IntermediateType::IStruct(index) => {
                let struct_ = module_data.structs.get_by_index(*index).unwrap();
                builder.load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
                struct_.copy_local_instructions(module, builder, compilation_ctx, module_data);
            }
            IntermediateType::IGenericStructInstance(index, types) => {
                let struct_ = module_data.structs.get_by_index(*index).unwrap();
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
                );
            }
            IntermediateType::IExternalUserData { .. } => todo!(),
            IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                // Nothing to be done, pointer is already correct
            }
            IntermediateType::ISigner => {
                panic!(r#"trying to introduce copy instructions for "signer" type"#)
            }
            IntermediateType::ITypeParameter(_) => {
                panic!("cannot copy a type parameter, expected a concrete type");
            }
            IntermediateType::IEnum(_) => todo!(),
        }
    }

    pub fn add_load_memory_to_local_instructions(
        &self,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        pointer: LocalId,
        memory: MemoryId,
    ) -> LocalId {
        match self {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::IAddress
            | IntermediateType::IStruct(_)
            | IntermediateType::IGenericStructInstance(_, _)
            | IntermediateType::IExternalUserData { .. }
            | IntermediateType::ISigner
            | IntermediateType::IVector(_)
            | IntermediateType::IRef(_)
            | IntermediateType::IMutRef(_) => {
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

                local
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

                local
            }
            IntermediateType::ITypeParameter(_) => {
                panic!("cannot load a type parameter, expected a concrete type");
            }
            IntermediateType::IEnum(_) => todo!(),
        }
    }

    /// Adds the instructions to load the value into a local variable
    /// Pops the next value from the stack and stores it in the a variable
    pub fn add_stack_to_local_instructions(
        &self,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
    ) -> LocalId {
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
            | IntermediateType::IExternalUserData { .. }
            | IntermediateType::IStruct(_)
            | IntermediateType::IEnum(_)
            | IntermediateType::IGenericStructInstance(_, _) => {
                let local = module.locals.add(ValType::I32);
                builder.local_set(local);
                local
            }
            IntermediateType::IU64 => {
                let local = module.locals.add(ValType::I64);
                builder.local_set(local);
                local
            }
            IntermediateType::ITypeParameter(_) => {
                panic!("cannot load a type parameter, expected a concrete type");
            }
        }
    }

    pub fn add_borrow_local_instructions(&self, builder: &mut InstrSeqBuilder, local: LocalId) {
        match self {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64
            | IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::ISigner
            | IntermediateType::IAddress
            | IntermediateType::IVector(_)
            | IntermediateType::IExternalUserData { .. }
            | IntermediateType::IStruct(_)
            | IntermediateType::IGenericStructInstance(_, _) => {
                builder.local_get(local);
            }
            IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                panic!("Cannot ImmBorrowLoc on a reference type");
            }
            IntermediateType::ITypeParameter(_) => {
                panic!("cannot borrow a type parameter, expected a concrete type");
            }
            IntermediateType::IEnum(_) => todo!(),
        }
    }

    pub fn add_read_ref_instructions(
        &self,
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
        module_data: &ModuleData,
    ) {
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
                let copy_f = RuntimeFunction::CopyU128.get(module, Some(compilation_ctx));
                builder.call(copy_f);
            }
            IntermediateType::IU256 | IntermediateType::IAddress => {
                let copy_f = RuntimeFunction::CopyU256.get(module, Some(compilation_ctx));
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
                );
            }
            IntermediateType::IStruct(index) => {
                let struct_ = module_data.structs.get_by_index(*index).unwrap();
                IStruct::copy_local_instructions(
                    struct_,
                    module,
                    builder,
                    compilation_ctx,
                    module_data,
                );
            }
            IntermediateType::IGenericStructInstance(index, types) => {
                let struct_ = module_data.structs.get_by_index(*index).unwrap();
                let struct_instance = struct_.instantiate(types);
                IStruct::copy_local_instructions(
                    &struct_instance,
                    module,
                    builder,
                    compilation_ctx,
                    module_data,
                );
            }
            IntermediateType::IExternalUserData { .. } => todo!(),
            IntermediateType::ISigner => {
                // Signer type is read-only, we push the pointer only
            }
            IntermediateType::IEnum(_) => todo!(),
            _ => panic!("Unsupported ReadRef type: {:?}", self),
        }
    }

    pub fn add_write_ref_instructions(
        &self,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        compilation_ctx: &CompilationContext,
    ) {
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
            | IntermediateType::IStruct(_)
            | IntermediateType::IGenericStructInstance(_, _) => {
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
                panic!("This type cannot be mutated: {:?}", self);
            }
            // TODO: Is this ok?
            IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                panic!("Cannot mutate a reference of a reference: {:?}", self);
            }
            IntermediateType::ITypeParameter(_) => {
                panic!("cannot write to a type parameter, expected a concrete type");
            }
            IntermediateType::IEnum(_) => todo!(),
            IntermediateType::IExternalUserData { .. } => todo!(),
        }
    }

    pub fn box_local_instructions(
        &self,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        compilation_ctx: &CompilationContext,
        local: LocalId,
    ) {
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
                    .i32_const(self.stack_data_size() as i32)
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
            | IntermediateType::IExternalUserData { .. }
            | IntermediateType::IStruct(_)
            | IntermediateType::IGenericStructInstance(_, _)
            | IntermediateType::IEnum(_) => {
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
                panic!("cannot box a type parameter, expected a concrete type");
            }
        }
    }

    pub fn load_equality_instructions(
        &self,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        compilation_ctx: &CompilationContext,
        module_data: &ModuleData,
    ) {
        match self {
            Self::IBool | Self::IU8 | Self::IU16 | Self::IU32 => {
                builder.binop(BinaryOp::I32Eq);
            }
            Self::IU64 => {
                builder.binop(BinaryOp::I64Eq);
            }
            Self::IU128 => IU128::equality(builder, module, compilation_ctx),
            Self::IU256 => IU256::equality(builder, module, compilation_ctx),
            Self::IAddress => IAddress::equality(builder, module, compilation_ctx),
            Self::ISigner => {
                // Signers can only be created by the VM and injected into the smart contract.
                // There can only be one signer, so if we find a situation where signers are
                // compared, we are comparing the same thing.
                builder.i32_const(1);
            }
            Self::IVector(inner) => IVector::equality(builder, module, compilation_ctx, inner),
            Self::IStruct(index) => {
                IStruct::equality(builder, module, compilation_ctx, module_data, *index)
            }
            Self::IGenericStructInstance(index, _) => {
                IStruct::equality(builder, module, compilation_ctx, module_data, *index)
            }
            Self::IEnum(_) => todo!(),
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
                    | IntermediateType::IStruct(_)
                    | IntermediateType::IGenericStructInstance(_, _) => {
                        builder.local_get(ptr1).local_get(ptr2);
                    }
                    IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                        panic!("found reference of reference");
                    }
                    IntermediateType::ITypeParameter(_) => {
                        panic!("Cannot compare a type parameter, expected a concrete type");
                    }
                    IntermediateType::IEnum(_) => todo!(),
                    IntermediateType::IExternalUserData { .. } => todo!(),
                }

                inner.load_equality_instructions(module, builder, compilation_ctx, module_data)
            }
            IntermediateType::ITypeParameter(_) => {
                panic!("cannot compare a type parameter, expected a concrete type");
            }
            IntermediateType::IExternalUserData { .. } => todo!(),
        }
    }

    pub fn load_not_equality_instructions(
        &self,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        compilation_ctx: &CompilationContext,
        module_data: &ModuleData,
    ) {
        self.load_equality_instructions(module, builder, compilation_ctx, module_data);
        builder.negate();
    }

    /// Returns true if the type is a stack type (the value is directly hanndled in wasm stack
    /// instead of handling a pointer), otherwise returns false.
    pub fn is_stack_type(&self) -> bool {
        match self {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64 => true,
            IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::IAddress
            | IntermediateType::ISigner
            | IntermediateType::IVector(_)
            | IntermediateType::IRef(_)
            | IntermediateType::IMutRef(_)
            | IntermediateType::IExternalUserData { .. }
            | IntermediateType::IStruct(_)
            | IntermediateType::IGenericStructInstance(_, _) => false,
            IntermediateType::ITypeParameter(_) => {
                panic!(
                    "cannot check if a type parameter is a stack type, expected a concrete type"
                );
            }
            IntermediateType::IEnum(_) => todo!(),
        }
    }
}

impl From<&IntermediateType> for ValType {
    fn from(value: &IntermediateType) -> Self {
        match value {
            IntermediateType::IU64 => ValType::I64, // If we change this, i64 will be stored as i32 for function arguments
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
            | IntermediateType::IExternalUserData { .. }
            | IntermediateType::IStruct(_)
            | IntermediateType::IGenericStructInstance(_, _)
            | IntermediateType::IEnum(_) => ValType::I32,
            IntermediateType::ITypeParameter(_) => {
                panic!("cannot convert a type parameter to a wasm type, expected a concrete type");
            }
        }
    }
}

impl From<IntermediateType> for ValType {
    fn from(value: IntermediateType) -> Self {
        Self::from(&value)
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
    ) -> Self {
        let arguments = arguments
            .0
            .iter()
            .map(|token| IntermediateType::try_from_signature_token(token, handles_map))
            .collect::<Result<Vec<IntermediateType>, anyhow::Error>>()
            // TODO: unwrap
            .unwrap();

        let returns = returns
            .0
            .iter()
            .map(|token| IntermediateType::try_from_signature_token(token, handles_map))
            .collect::<Result<Vec<IntermediateType>, anyhow::Error>>()
            // TODO: unwrap
            .unwrap();

        Self { arguments, returns }
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

    pub fn get_argument_wasm_types(&self) -> Vec<ValType> {
        self.arguments.iter().map(ValType::from).collect()
    }
}
