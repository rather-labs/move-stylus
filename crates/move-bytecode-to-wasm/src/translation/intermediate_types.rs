use address::IAddress;
use boolean::IBool;
use heap_integers::{IU128, IU256};
use move_binary_format::file_format::{Signature, SignatureToken};
use simple_integers::{IU8, IU16, IU32, IU64};
use vector::IVector;
use walrus::{
    InstrSeqBuilder, LocalId, MemoryId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use crate::CompilationContext;
use crate::{runtime::RuntimeFunction, wasm_builder_extensions::WasmBuilderExtension};
pub mod address;
pub mod boolean;
pub mod heap_integers;
pub mod reference;
pub mod signer;
pub mod simple_integers;
pub mod vector;

#[derive(Clone, PartialEq, Debug)]
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
            | IntermediateType::IMutRef(_) => 4,
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
            | IntermediateType::ISigner => {
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
            IntermediateType::IVector(_) => {
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
    }

    pub fn copy_local_instructions(
        &self,
        module: &mut Module,
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
                //TODO: move this to a runtime function
                IVector::copy_local_instructions(inner_type, module, builder, compilation_ctx);
            }
            IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                // Nothing to be done, pointer is already correct
            }
            IntermediateType::ISigner => {
                panic!(r#"trying to introduce copy instructions for "signer" type"#)
            }
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
            | IntermediateType::IMutRef(_) => {
                let local = module.locals.add(ValType::I32);
                builder.local_set(local);
                local
            }
            IntermediateType::IU64 => {
                let local = module.locals.add(ValType::I64);
                builder.local_set(local);
                local
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
            | IntermediateType::IVector(_) => {
                builder.local_get(local);
            }
            IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                panic!("Cannot ImmBorrowLoc on a reference type");
            }
        }
    }

    pub fn add_read_ref_instructions(&self, builder: &mut InstrSeqBuilder, memory: MemoryId) {
        builder.load(
            memory,
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
                    memory,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            }
            IntermediateType::IU64 => {
                builder.load(
                    memory,
                    LoadKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            }
            IntermediateType::IVector(_)
            | IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::ISigner
            | IntermediateType::IAddress => {
                // No load needed, pointer is already correct
            }
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
            IntermediateType::IVector(_) => {
                // Since the memory needed for vectors might differ, we don't overwrite it.
                // We update the inner pointer to point to the location where the new vector is already allocated.
                let src_ptr = module.locals.add(ValType::I32);
                let ref_ptr = module.locals.add(ValType::I32);

                // Swap pointers order in the stack
                builder
                    .local_set(ref_ptr)
                    .local_set(src_ptr)
                    .local_get(ref_ptr)
                    .local_get(src_ptr);

                // Store src_ptr at ref_ptr
                // Now the inner pointer is updated to point to the new vector
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
            _ => unreachable!(),
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
            | IntermediateType::IMutRef(_) => {
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
        }
    }

    pub fn load_equality_instructions(
        &self,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        compilation_ctx: &CompilationContext,
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
                    | IntermediateType::IVector(_) => {
                        builder.local_get(ptr1).local_get(ptr2);
                    }
                    IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                        panic!("found reference of reference");
                    }
                }

                inner.load_equality_instructions(module, builder, compilation_ctx)
            }
        }
    }

    pub fn load_not_equality_instructions(
        &self,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        compilation_ctx: &CompilationContext,
    ) {
        self.load_equality_instructions(module, builder, compilation_ctx);
        builder.negate();
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
            | IntermediateType::IMutRef(_) => ValType::I32,
        }
    }
}

impl From<IntermediateType> for ValType {
    fn from(value: IntermediateType) -> Self {
        Self::from(&value)
    }
}

impl TryFrom<&SignatureToken> for IntermediateType {
    // TODO: Change when handling errors better
    type Error = anyhow::Error;

    fn try_from(value: &SignatureToken) -> Result<Self, Self::Error> {
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
                let itoken = Self::try_from(token.as_ref())?;
                Ok(IntermediateType::IVector(Box::new(itoken)))
            }
            SignatureToken::Reference(token) => {
                let itoken = Self::try_from(token.as_ref())?;
                Ok(IntermediateType::IRef(Box::new(itoken)))
            }
            SignatureToken::MutableReference(token) => {
                let itoken = Self::try_from(token.as_ref())?;
                Ok(IntermediateType::IMutRef(Box::new(itoken)))
            }
            _ => Err(anyhow::anyhow!("Unsupported signature token: {value:?}")),
        }
    }
}

pub struct ISignature {
    pub arguments: Vec<IntermediateType>,
    pub returns: Vec<IntermediateType>,
}

impl ISignature {
    pub fn from_signatures(arguments: &Signature, returns: &Signature) -> Self {
        let arguments = arguments
            .0
            .iter()
            .map(|token| token.try_into())
            .collect::<Result<Vec<IntermediateType>, anyhow::Error>>()
            // TODO: unwrap
            .unwrap();

        let returns = returns
            .0
            .iter()
            .map(|token| token.try_into())
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
