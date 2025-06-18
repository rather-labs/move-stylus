use crate::{CompilationContext, translation::intermediate_types::IntermediateType};

mod function_encoding;
pub(crate) mod packing;
pub mod public_function;
mod unpacking;

impl IntermediateType {
    /// According to the formal specification of the encoding, a tuple (T1,...,Tk) is dynamic if
    /// Ti is dynamic for some 1 <= i <= k.
    ///
    /// Structs are encoded as a tuple of its fields, so, if any field is dynamic, then the whole
    /// struct is dynamic.
    ///
    /// According to documentation, dynamic types are:
    /// - bytes
    /// - string
    /// - T[] for any T
    /// - T[k] for any dynamic T and any k >= 0
    /// - (T1,...,Tk) if Ti is dynamic for some 1 <= i <= k
    ///
    /// For more information:
    /// https://docs.soliditylang.org/en/develop/abi-spec.html#formal-specification-of-the-encoding
    pub fn solidity_abi_encode_is_dynamic(&self, compilation_ctx: &CompilationContext) -> bool {
        match self {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64
            | IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::IAddress => false,
            IntermediateType::IVector(_) => true,
            IntermediateType::IStruct(index) => {
                let struct_ = compilation_ctx.get_struct_by_index(*index).unwrap();
                struct_.solidity_abi_encode_is_dynamic(compilation_ctx)
            }
            IntermediateType::ISigner => panic!("signer is not abi econdable"),
            IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                panic!("found reference inside struct")
            }
        }
    }
}
