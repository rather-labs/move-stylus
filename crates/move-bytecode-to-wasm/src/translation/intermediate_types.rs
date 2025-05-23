use address::IAddress;
use boolean::IBool;
use heap_integers::{IU128, IU256};
use move_binary_format::file_format::{Signature, SignatureToken};
use simple_integers::{IU8, IU16, IU32, IU64};
use vector::IVector;
use walrus::{
    InstrSeqBuilder, LocalId, MemoryId, Module, ValType,
    ir::{LoadKind, MemArg, StoreKind},
};

use crate::CompilationContext;

pub mod address;
pub mod boolean;
pub mod heap_integers;
pub mod imm_reference;
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
            | IntermediateType::IRef(_) => 4,
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
            IntermediateType::IRef(_) => {
                panic!("cannot load a constant for a reference type");
            }
        }
    }

    pub fn store_local_instructions(
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
            | IntermediateType::IU32
            | IntermediateType::IU64 => {
                let (valtype, storekind) = match self {
                    IntermediateType::IU64 => (ValType::I64, StoreKind::I64 { atomic: false }),
                    _ => (ValType::I32, StoreKind::I32 { atomic: false }),
                };
                // The value is on top of the stack. We store it in an aux variable
                let value = module.locals.add(valtype);
                builder.local_set(value);

                // The local will actually hold a pointer to the value, which is stored in memory
                builder
                    .i32_const(self.stack_data_size() as i32)
                    .call(compilation_ctx.allocator)
                    .local_tee(local)
                    .local_get(value)
                    .store(
                        compilation_ctx.memory_id,
                        storekind,
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
            | IntermediateType::IRef(_) => {
                // The address is on top of the stack. We store it in the local variable directly.
                builder.local_set(local);
            }
        }
    }

    pub fn move_local_instructions(
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
            | IntermediateType::IU32
            | IntermediateType::IU64 => {
                // The local holds a pointer to the value, which is stored in memory
                // The value is loaded into the stack
                builder.local_get(local);
                builder.load(
                    compilation_ctx.memory_id,
                    match self {
                        IntermediateType::IU64 => LoadKind::I64 { atomic: false },
                        _ => LoadKind::I32 { atomic: false },
                    },
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
            | IntermediateType::IRef(_) => {
                // For heap types we just forward the pointer
                builder.local_get(local);
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
        match self {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64 => {
                builder.local_get(local);
                builder.load(
                    compilation_ctx.memory_id,
                    match self {
                        IntermediateType::IU64 => LoadKind::I64 { atomic: false },
                        _ => LoadKind::I32 { atomic: false },
                    },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            }
            IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::IAddress
            | IntermediateType::IRef(_) => {
                builder.local_get(local);
            }
            IntermediateType::IVector(inner) => {
                IVector::copy_local_instructions(inner, module, builder, compilation_ctx, local);
            }
            // `signer` type is not copy, this should never happen
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
            | IntermediateType::IRef(_) => {
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
            | IntermediateType::IRef(_) => {
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

    pub fn add_borrow_local_instructions(
        &self,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        compilation_ctx: &CompilationContext,
        local: LocalId,
    ) {
        match self {
            // For primitives, we copy the value in memory and return a pointer to it
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64 => {
                let size = self.stack_data_size() as i32;
                let ptr = module.locals.add(ValType::I32);

                builder.i32_const(size);
                builder.call(compilation_ctx.allocator);
                builder.local_tee(ptr);

                builder.local_get(local);
                builder.store(
                    compilation_ctx.memory_id,
                    match self {
                        IntermediateType::IU64 => StoreKind::I64 { atomic: false },
                        _ => StoreKind::I32 { atomic: false },
                    },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

                builder.local_get(ptr); // leave the pointer on the stack as &T
            }

            // Heap allocated: just forward the existing pointer
            IntermediateType::IVector(_)
            | IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::ISigner
            | IntermediateType::IAddress => {
                builder.local_get(local);
            }
            IntermediateType::IRef(_) => {
                panic!("Cannot ImmBorrowLoc on a reference type");
            }
        }
    }

    pub fn add_read_ref_instructions(&self, builder: &mut InstrSeqBuilder, memory: MemoryId) {
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
}

impl From<&IntermediateType> for ValType {
    fn from(value: &IntermediateType) -> Self {
        match value {
            IntermediateType::IU64 => ValType::I64,
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::IAddress
            | IntermediateType::ISigner
            | IntermediateType::IVector(_)
            | IntermediateType::IRef(_) => ValType::I32,
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
