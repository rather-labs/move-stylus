use address::IAddress;
use boolean::IBool;
use heap_integers::{IU128, IU256};
use move_binary_format::file_format::{Signature, SignatureToken};
use simple_integers::{IU8, IU16, IU32, IU64};
use vector::IVector;
use walrus::{
    FunctionId, InstrSeqBuilder, LocalId, MemoryId, Module, ValType,
    ir::{LoadKind, MemArg},
};

pub mod address;
pub mod boolean;
pub mod heap_integers;
pub mod simple_integers;
pub mod vector;

#[derive(Clone)]
pub enum IntermediateType {
    IBool,
    IU8,
    IU16,
    IU32,
    IU64,
    IU128,
    IU256,
    IAddress,
    IVector(Box<IntermediateType>),
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
            | IntermediateType::IVector(_) => ValType::I32,
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
            | IntermediateType::IVector(_) => 4,
        }
    }

    /// Adds the instructions to load the constant into the local variable
    /// Pops the first n elements from `bytes` iterator and stores them in memory, removing them from the vector
    ///
    /// For heap and reference types, the actual value is stored in memory and a pointer is returned
    pub fn load_constant_instructions(
        &self,
        module: &mut Module,
        function_id: FunctionId,
        bytes: &mut std::vec::IntoIter<u8>,
        allocator: FunctionId,
        memory: MemoryId,
    ) {
        match self {
            IntermediateType::IBool => {
                IBool::load_constant_instructions(module, function_id, bytes, allocator, memory)
            }
            IntermediateType::IU8 => {
                IU8::load_constant_instructions(module, function_id, bytes, allocator, memory)
            }
            IntermediateType::IU16 => {
                IU16::load_constant_instructions(module, function_id, bytes, allocator, memory)
            }
            IntermediateType::IU32 => {
                IU32::load_constant_instructions(module, function_id, bytes, allocator, memory)
            }
            IntermediateType::IU64 => {
                IU64::load_constant_instructions(module, function_id, bytes, allocator, memory)
            }
            IntermediateType::IU128 => {
                IU128::load_constant_instructions(module, function_id, bytes, allocator, memory)
            }
            IntermediateType::IU256 => {
                IU256::load_constant_instructions(module, function_id, bytes, allocator, memory)
            }
            IntermediateType::IAddress => {
                IAddress::load_constant_instructions(module, function_id, bytes, allocator, memory)
            }
            IntermediateType::IVector(inner) => IVector::load_constant_instructions(
                inner,
                module,
                function_id,
                bytes,
                allocator,
                memory,
            ),
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
            | IntermediateType::IVector(_) => {
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

    pub fn copy_loc_instructions(
        &self,
        module: &mut Module,
        function_id: FunctionId,
        _allocator: FunctionId,
        _memory: MemoryId,
        source_ptr: LocalId,
    ) -> LocalId {
        let builder = &mut get_function_body_builder(module, function_id);
    
        match self {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64 => {
                builder.local_get(source_ptr);
                source_ptr
            }
    
            IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::IAddress => {
                // these are on the heap, pointer must be copied
                let local = module.locals.add(ValType::I32);
                builder.local_get(source_ptr);
                builder.local_set(local);
                local
            }
    
            IntermediateType::IVector(inner) => {
                IVector::copy_loc_instructions(
                    inner,
                    module,
                    function_id,
                    _allocator,
                    _memory,
                    source_ptr,
                )
            }
    
            _ => unimplemented!("copy for {:?}", self),
        }
    }    
}

pub trait SignatureTokenToIntermediateType {
    fn to_intermediate_type(&self) -> IntermediateType;
}

impl SignatureTokenToIntermediateType for SignatureToken {
    fn to_intermediate_type(&self) -> IntermediateType {
        match self {
            SignatureToken::Bool => IntermediateType::IBool,
            SignatureToken::U8 => IntermediateType::IU8,
            SignatureToken::U16 => IntermediateType::IU16,
            SignatureToken::U32 => IntermediateType::IU32,
            SignatureToken::U64 => IntermediateType::IU64,
            SignatureToken::U128 => IntermediateType::IU128,
            SignatureToken::U256 => IntermediateType::IU256,
            SignatureToken::Address => IntermediateType::IAddress,
            SignatureToken::Vector(token) => {
                IntermediateType::IVector(Box::new(token.to_intermediate_type()))
            }
            _ => panic!("Unsupported signature token: {:?}", self),
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
            .map(|token| token.to_intermediate_type())
            .collect();
        let returns = returns
            .0
            .iter()
            .map(|token| token.to_intermediate_type())
            .collect();

        Self { arguments, returns }
    }

    pub fn get_return_wasm_types(&self) -> Vec<ValType> {
        self.returns.iter().map(|t| t.to_wasm_type()).collect()
    }

    pub fn get_argument_wasm_types(&self) -> Vec<ValType> {
        self.arguments.iter().map(|t| t.to_wasm_type()).collect()
    }
}
