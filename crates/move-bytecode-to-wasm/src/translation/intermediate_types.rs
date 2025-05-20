use address::IAddress;
use boolean::IBool;
use heap_integers::{IU128, IU256};
use move_binary_format::file_format::{Signature, SignatureToken};
use simple_integers::{IU8, IU16, IU32, IU64};
use vector::IVector;
use walrus::{
    FunctionId, InstrSeqBuilder, LocalId, MemoryId, ModuleLocals, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind, UnaryOp},
};

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
    /// Returns the wasm type that represents the intermediate type
    /// For heap or reference types, it references a pointer to memory
    pub fn to_wasm_type(&self) -> ValType {
        match self {
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
        module_locals: &mut ModuleLocals,
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
        allocator: FunctionId,
        memory: MemoryId,
    ) {
        match self {
            IntermediateType::IBool => IBool::load_constant_instructions(builder, bytes),
            IntermediateType::IU8 => IU8::load_constant_instructions(builder, bytes),
            IntermediateType::IU16 => IU16::load_constant_instructions(builder, bytes),
            IntermediateType::IU32 => IU32::load_constant_instructions(builder, bytes),
            IntermediateType::IU64 => IU64::load_constant_instructions(builder, bytes),
            IntermediateType::IU128 => {
                IU128::load_constant_instructions(module_locals, builder, bytes, allocator, memory)
            }
            IntermediateType::IU256 => {
                IU256::load_constant_instructions(module_locals, builder, bytes, allocator, memory)
            }
            IntermediateType::IAddress => IAddress::load_constant_instructions(
                module_locals,
                builder,
                bytes,
                allocator,
                memory,
            ),
            IntermediateType::ISigner => panic!("signer type can't be loaded as a constant"),
            IntermediateType::IVector(inner) => IVector::load_constant_instructions(
                inner,
                module_locals,
                builder,
                bytes,
                allocator,
                memory,
            ),
            IntermediateType::IRef(_) => {
                panic!("cannot load a constant for a reference type");
            }
        }
    }

    pub fn copy_loc_instructions(
        &self,
        module_locals: &mut ModuleLocals,
        builder: &mut InstrSeqBuilder,
        allocator: FunctionId,
        memory: MemoryId,
        local: LocalId,
    ) {
        match self {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64
            | IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::IAddress
            | IntermediateType::IRef(_) => {
                builder.local_get(local);
            }
            IntermediateType::IVector(inner) => {
                IVector::copy_loc_instructions(
                    inner,
                    module_locals,
                    builder,
                    allocator,
                    memory,
                    local,
                );
            }
            // `signer` type is not copy, this should never happen
            IntermediateType::ISigner => {
                panic!(r#"trying to introduce copy instructions for "signer" type"#)
            }
        }
    }

    pub fn add_load_memory_to_local_instructions(
        &self,
        module_locals: &mut ModuleLocals,
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
                let local = module_locals.add(ValType::I32);

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
                let local = module_locals.add(ValType::I64);

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
        module_locals: &mut ModuleLocals,
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
                let local = module_locals.add(ValType::I32);
                builder.local_set(local);
                local
            }
            IntermediateType::IU64 => {
                let local = module_locals.add(ValType::I64);
                builder.local_set(local);
                local
            }
        }
    }

    pub fn add_imm_borrow_loc_instructions(
        &self,
        module_locals: &mut ModuleLocals,
        builder: &mut InstrSeqBuilder,
        allocator: FunctionId,
        memory: MemoryId,
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
                let ptr = module_locals.add(ValType::I32);

                builder.i32_const(size);
                builder.call(allocator);
                builder.local_tee(ptr);

                builder.local_get(local);
                builder.store(
                    memory,
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

    pub fn add_vec_imm_borrow_instructions(
        &self,
        module_locals: &mut ModuleLocals,
        builder: &mut InstrSeqBuilder,
        allocator: FunctionId,
        memory: MemoryId,
    ) {
        let size = self.stack_data_size() as i32;
        let index_i64 = module_locals.add(ValType::I64); // referenced element index
        builder.local_set(index_i64); // index is on top of stack (as i64)

        // Trap if index > u32::MAX
        builder.block(None, |block| {
            block
                .local_get(index_i64)
                .i64_const(0xFFFF_FFFF)
                .binop(BinaryOp::I64LeU);
            block.br_if(block.id());
            block.unreachable();
        });

        //  Cast index to i32
        let index_i32 = module_locals.add(ValType::I32);
        builder
            .local_get(index_i64)
            .unop(UnaryOp::I32WrapI64)
            .local_set(index_i32);

        // Set vector base address
        let vector_address = module_locals.add(ValType::I32);
        builder.local_set(vector_address);

        // Trap if index >= length
        builder.block(None, |block| {
            block
                .local_get(vector_address)
                .load(
                    memory,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .local_get(index_i32)
                .binop(BinaryOp::I32GtU);
            block.br_if(block.id());
            block.unreachable();
        });

        // Compute element
        let element_local = module_locals.add(ValType::I32);
        builder
            .local_get(vector_address)
            .i32_const(4)
            .binop(BinaryOp::I32Add)
            .local_get(index_i32)
            .i32_const(size)
            .binop(BinaryOp::I32Mul)
            .binop(BinaryOp::I32Add)
            .local_set(element_local);

        match self {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64 => {
                builder.local_get(element_local);
            }

            IntermediateType::IVector(_)
            | IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::ISigner
            | IntermediateType::IAddress => {
                builder.local_get(element_local).load(
                    memory,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            }

            IntermediateType::IRef(_) => {
                panic!("Cannot VecImmBorrow an existing reference type");
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
        self.arguments.iter().map(|t| t.to_wasm_type()).collect()
    }
}
