use walrus::{FunctionId, Module};

use crate::CompilationContext;

mod integers;

#[derive(PartialEq)]
pub enum RuntimeFunction {
    // Integer operations
    HeapIntSum,
    AddU32,
    AddU64,
    CheckOverflowU8U16,
    DowncastU64ToU32,
    DowncastU128U256ToU32,
    DowncastU128U256ToU64,
}

impl RuntimeFunction {
    pub fn name(&self) -> &'static str {
        match self {
            RuntimeFunction::HeapIntSum => "heap_integer_add",
            RuntimeFunction::AddU32 => "add_u32",
            RuntimeFunction::AddU64 => "add_u64",
            RuntimeFunction::CheckOverflowU8U16 => "check_overflow_u8_u16",
            RuntimeFunction::DowncastU64ToU32 => "downcast_u64_to_u32",
            RuntimeFunction::DowncastU128U256ToU32 => "downcast_u128_u256_to_u32",
            RuntimeFunction::DowncastU128U256ToU64 => "downcast_u128_u256_to_u64",
        }
    }

    /// Links the function into the module and returns its id. If the function is already present
    /// it just resunts the id.
    ///
    /// This funciton is idempotent.
    pub fn link_and_get_id(
        &self,
        module: &mut Module,
        compilation_ctx: Option<&CompilationContext>,
    ) -> FunctionId {
        if let Some(function) = module.funcs.by_name(self.name()) {
            function
        } else {
            match (self, compilation_ctx) {
                (RuntimeFunction::HeapIntSum, Some(ctx)) => {
                    integers::heap_integers_add(module, ctx)
                }
                (RuntimeFunction::AddU32, _) => integers::add_u32(module),
                (RuntimeFunction::AddU64, _) => integers::add_u64(module),
                (RuntimeFunction::CheckOverflowU8U16, _) => integers::check_overflow_u8_u16(module),
                (RuntimeFunction::DowncastU64ToU32, _) => integers::downcast_u64_to_u32(module),
                (RuntimeFunction::DowncastU128U256ToU32, Some(ctx)) => {
                    integers::downcast_u128_u256_to_u32(module, ctx)
                }
                (RuntimeFunction::DowncastU128U256ToU64, Some(ctx)) => {
                    integers::downcast_u128_u256_to_u64(module, ctx)
                }
                _ => panic!(
                    r#"there was an error linking "{}" function, missing compilation context?"#,
                    self.name()
                ),
            }
        }
    }
}
