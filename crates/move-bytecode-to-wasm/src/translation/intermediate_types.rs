use address::IAddress;
use boolean::IBool;
use heap_integers::{IU128, IU256};
use move_binary_format::file_format::{Signature, SignatureToken};
use simple_integers::{IU8, IU16, IU32, IU64};
use walrus::{FunctionId, MemoryId, Module, ValType};

use crate::abi_types::{Packable, SolName, Unpackable};

pub mod address;
pub mod boolean;
pub mod heap_integers;
pub mod simple_integers;

pub trait IntermediateType {
    /// Returns the wasm type that represents the intermediate type
    /// For heap or reference types, it references a pointer to memory
    fn to_wasm_type(&self) -> ValType;

    /// Adds the instructions to load the constant into the local variable
    /// For heap and reference types, the actual value is stored in memory and a pointer is returned
    fn load_constant_instructions(
        &self,
        module: &mut Module,
        function_id: FunctionId,
        bytes: &[u8],
        allocator: FunctionId,
        memory: MemoryId,
    );
}

pub trait SignatureTokenToIntermediateType {
    fn to_intermediate_type(&self) -> Box<dyn IParam>;
}

impl SignatureTokenToIntermediateType for SignatureToken {
    fn to_intermediate_type(&self) -> Box<dyn IParam> {
        match self {
            SignatureToken::Bool => Box::new(IBool),
            SignatureToken::U8 => Box::new(IU8),
            SignatureToken::U16 => Box::new(IU16),
            SignatureToken::U32 => Box::new(IU32),
            SignatureToken::U64 => Box::new(IU64),
            SignatureToken::U128 => Box::new(IU128),
            SignatureToken::U256 => Box::new(IU256),
            SignatureToken::Address => Box::new(IAddress),
            _ => panic!("Unsupported signature token: {:?}", self),
        }
    }
}

pub trait IParam: IntermediateType + SolName + Packable + Unpackable {}

pub struct ISignature {
    pub arguments: Vec<Box<dyn IParam>>,
    pub returns: Vec<Box<dyn IParam>>,
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
