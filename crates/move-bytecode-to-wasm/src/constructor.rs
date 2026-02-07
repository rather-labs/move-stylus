use walrus::{
    FunctionBuilder, FunctionId as WalrusFunctionId, Module, ValType,
    ir::{MemArg, StoreKind},
};

use crate::{
    CompilationContext,
    abi_types::{error::AbiError, public_function::PublicFunction},
    error::{CompilationError, ICEError, ICEErrorKind},
    hostio::host_functions,
    runtime::{RuntimeFunction, error::RuntimeFunctionError},
    translation::table::FunctionTable,
    utils::keccak_string_to_memory,
    vm_handled_types::{VmHandledType, error::VmHandledTypeError, tx_context::TxContext},
};

#[derive(Debug, thiserror::Error)]
pub enum ConstructorError {
    #[error("abi error ocurred while processing a native function")]
    Abi(#[from] AbiError),

    #[error("an error ocurred while generating a runtime function's code")]
    RuntimeFunction(#[from] RuntimeFunctionError),

    #[error("init function not found in table")]
    InitFunctionNotFound,

    #[error("init function WASM id not found in table")]
    InitFunctionWasmIdNotFound,

    #[error("an error ocurred while injecting a vm handled type")]
    VmHandledType(#[from] VmHandledTypeError),
}

impl From<ConstructorError> for CompilationError {
    fn from(value: ConstructorError) -> Self {
        CompilationError::ICE(ICEError::new(ICEErrorKind::Constructor(value)))
    }
}

/// Injects the constructor as a public function in the module, which will be accesible via the entrypoint router.
pub fn inject_constructor<'a>(
    function_table: &mut FunctionTable,
    module: &mut Module,
    compilation_ctx: &'a CompilationContext,
    public_functions: &mut Vec<PublicFunction<'a>>,
) -> Result<(), ConstructorError> {
    if let Some(ref init_id) = compilation_ctx.root_module_data.functions.init {
        let wasm_init_fn = function_table
            .get_by_function_id(init_id)
            .ok_or(ConstructorError::InitFunctionNotFound)?
            .wasm_function_id
            .ok_or(ConstructorError::InitFunctionWasmIdNotFound)?;

        let constructor_fn_id = build_constructor(module, compilation_ctx, wasm_init_fn)?;

        public_functions.push(PublicFunction::new(
            constructor_fn_id,
            "constructor",
            &compilation_ctx.empty_signature,
            compilation_ctx,
        )?);
    };

    Ok(())
}

/// Builds the constructor function.
///
/// This function performs the following actions:
/// 1. Verifies whether the constructor has been invoked before using a storage key guard.
/// 2. If it hasn't, it calls the `init()` function.
/// 3. Records in persistent storage that the constructor has been executed.
///
/// This ensures the constructor logic executes only once and safely initializes global storage.
pub fn build_constructor(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    init: WalrusFunctionId,
) -> Result<WalrusFunctionId, ConstructorError> {
    // Flag to indicate if the constructor has been called.
    // This is what we are going to be storing in the storage.
    const FLAG: i32 = 1;

    // Host function for checking if all bytes are zero
    let is_zero_fn = RuntimeFunction::IsZero.get(module, Some(compilation_ctx), None)?;

    // Host functions for storage operations
    let (storage_load_fn, _) = host_functions::storage_load_bytes32(module);
    let (storage_cache_fn, _) = host_functions::storage_cache_bytes32(module);
    let (flush_cache_fn, _) = host_functions::storage_flush_cache(module);

    // Allocate local variables to hold memory pointers
    let key_ptr = module.locals.add(ValType::I32); // Pointer for the storage key
    let value_ptr = module.locals.add(ValType::I32); // Pointer to store flag

    // Define the constructor function with no parameters or return values
    let mut function = FunctionBuilder::new(&mut module.types, &[], &[]);
    let mut builder = function.func_body();

    // ptr to storage key
    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_set(key_ptr);

    // Init key is the keccak256 hash of "init_key"
    keccak_string_to_memory(&mut builder, compilation_ctx, "init_key", key_ptr);

    // ptr to value
    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_set(value_ptr);

    // Read from storage into value_ptr
    builder
        .local_get(key_ptr)
        .local_get(value_ptr)
        .call(storage_load_fn);

    // Check if the storage is empty, else it has been initialized
    builder.local_get(value_ptr).i32_const(32).call(is_zero_fn);

    // If storage has not been initialized, proceed with initialization
    let mut inner_result: Result<(), VmHandledTypeError> = Ok(());
    builder.if_else(
        None,
        |then| {
            // Inject TxContext as last argument
            inner_result = TxContext::inject(then, module, compilation_ctx);

            // Call the `init` function
            then.call(init);

            // Write the flag at value_ptr
            then.local_get(value_ptr).i32_const(FLAG).store(
                compilation_ctx.memory_id,
                StoreKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

            // Cache the flag
            then.local_get(key_ptr)
                .local_get(value_ptr)
                .call(storage_cache_fn);

            // Flush storage to persist the flag
            then.i32_const(1).call(flush_cache_fn);
        },
        |_else| {
            // Constructor already called → do nothing
        },
    );

    inner_result?;

    // Finalize and insert the function into the module
    Ok(function.finish(vec![], &mut module.funcs))
}
