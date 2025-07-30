use crate::CompilationContext;
use crate::UserDefinedType;
use crate::abi_types::vm_handled_datatypes::TxContext;
use crate::hostio::host_functions;
use crate::translation::intermediate_types::IntermediateType;
use crate::translation::table::FunctionId;
use move_binary_format::file_format::{
    Ability, AbilitySet, CompiledModule, DatatypeHandleIndex, FunctionDefinition, Signature,
    SignatureToken, Visibility,
};
use std::collections::HashMap;
use walrus::{
    FunctionBuilder, FunctionId as WalrusFunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

pub fn inject_constructor(
    module: &mut Module,
    allocator_func: WalrusFunctionId,
    compilation_ctx: &CompilationContext,
    init: Option<WalrusFunctionId>,
) -> WalrusFunctionId {
    let (storage_load_bytes32_function, _) = host_functions::storage_load_bytes32(module);
    let (storage_cache_bytes32_function, _) = host_functions::storage_cache_bytes32(module);
    let (storage_flush_cache_function, _) = host_functions::storage_flush_cache(module);
    let (emit_log_function, _) = host_functions::emit_log(module);

    // Allocate locals for key and value pointers
    let key_ptr = module.locals.add(ValType::I32);
    let value_ptr = module.locals.add(ValType::I32);
    let dest_ptr = module.locals.add(ValType::I32);

    let mut function = FunctionBuilder::new(&mut module.types, &[], &[]);
    let mut builder = function.func_body();

    // TODO: how could we avoid allocating more than once, in case the constructor is called again?
    // anyway this should be an edge case

    // Allocate memory
    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_set(key_ptr);

    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_set(dest_ptr);

    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_set(value_ptr);

    // Initialize key memory with 32 bytes of zero
    for offset in (0..32).step_by(4) {
        builder.local_get(key_ptr);
        builder.i32_const(0);
        builder.store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg { align: 0, offset },
        );
    }

    // Read from storage
    builder
        .local_get(key_ptr)
        .local_get(dest_ptr)
        .call(storage_load_bytes32_function); // Reads at key_ptr and writes at dest_ptr

    builder.local_get(dest_ptr);
    builder.i32_const(32);
    builder.i32_const(0);
    builder.call(emit_log_function);

    // Load the value writen at dest_ptr and check if its zero
    builder
        .local_get(dest_ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .i32_const(0)
        .binop(BinaryOp::I32Eq);

    // If zero, then the memory has not been written yet, meaning the constructor has not been called yet
    builder.if_else(
        None,
        |then| {
            // Write 1 to memory offset 0
            then.local_get(value_ptr).i32_const(1).store(
                compilation_ctx.memory_id,
                StoreKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

            // If there is an init function, call it
            if let Some(init_id) = init {
                let init_type = module.funcs.get(init_id).ty();
                let params = module.types.get(init_type).params();

                // If init expects an OTW, push dummy OTW
                if params.len() == 2 {
                    then.i32_const(0);
                }

                // Inject TxContext
                TxContext::inject_tx_context(then, allocator_func);

                // Call init
                then.call(init_id);
            }

            // Write (key, value) to storage
            then.local_get(key_ptr).local_get(value_ptr).call(storage_cache_bytes32_function);

            // Flush storage cache
            then.i32_const(1).call(storage_flush_cache_function);
        },
        |else_| {

            else_.unreachable();

            // TODO: remove this, is just for testing.

            // else_.local_get(value_ptr).i32_const(2).store(
            //     compilation_ctx.memory_id,
            //     StoreKind::I32 { atomic: false },
            //     MemArg {
            //         align: 0,
            //         offset: 0,
            //     },
            // );

            //  // Write (key, value) to storage
            //  else_.local_get(key_ptr).local_get(value_ptr).call(storage_cache_bytes32_function);

            //  // Flush storage cache
            //  else_.i32_const(1).call(storage_flush_cache_function);
        },
    );

    function.finish(vec![], &mut module.funcs)
}

const INIT_FUNCTION_NAME: &str = "init";

// The init() function is a special function that is called once when the module is first deployed,
// so it is a good place to put the code that initializes module's objects and sets up the environment and configuration.
//
// For the init() function to be considered valid, it must adhere to the following requirements:
// 1. It must be named `init`.
// 2. It must be private.
// 3. It must have &TxContext or &mut TxContext as its last argument, with an optional One Time Witness (OTW) as its first argument.
// 4. It must not return any values.
//
// fun init(ctx: &TxContext) { /* ... */}
// fun init(otw: OTW, ctx: &mut TxContext) { /* ... */ }
//

/// Checks if the given function (by index) is a valid `init` function.
// TODO: Note that we currently trigger a panic if a function named 'init' fails to satisfy certain criteria to qualify as a constructor.
// This behavior is not enforced by the move compiler itself.
pub fn is_init(
    function_id: FunctionId,
    move_function_arguments: &Signature,
    move_function_return: &Signature,
    function_def: &FunctionDefinition,
    datatype_handles_map: &HashMap<DatatypeHandleIndex, UserDefinedType>,
    module: &CompiledModule,
) -> bool {
    // Must be named `init`
    if function_id.identifier != INIT_FUNCTION_NAME {
        return false;
    }

    // Must be private
    assert_eq!(
        function_def.visibility,
        Visibility::Private,
        "init() functions must be private"
    );

    // Must have 1 or 2 arguments
    let arg_count = move_function_arguments.len();
    assert!(
        (1..=2).contains(&arg_count),
        "init() functions must have 1 or 2 arguments"
    );

    // Check TxContext in the last argument
    let is_tx_context_ref = move_function_arguments
        .0
        .last()
        .map(|last| {
            matches!(
                IntermediateType::try_from_signature_token(last, datatype_handles_map).unwrap(),
                IntermediateType::IRef(inner) | IntermediateType::IMutRef(inner)
                    if matches!(
                        inner.as_ref(),
                        IntermediateType::IExternalUserData { module_id, identifier }
                            if TxContext::struct_is_tx_context(module_id, identifier)
                    )
            )
        })
        .unwrap_or(false);

    assert!(
        is_tx_context_ref,
        "Last argument must be &TxContext or &mut TxContext"
    );

    // Check OTW if 2 arguments
    if arg_count == 2 {
        assert!(
            is_one_time_witness(module, &move_function_arguments.0[0]),
            "First argument must be a valid one-time witness type"
        );
    }

    // Must not return any values
    assert!(
        move_function_return.is_empty(),
        "init() functions must return no values"
    );

    true
}

/// Checks if the given signature token is a one-time witness type.
//
// OTW (One-time witness) types are structs with the following requirements:
// i. Their name is the upper-case version of the module's name.
// ii. They have no fields (or a single boolean field).
// iii. They have no type parameters.
// iv. They have only the 'drop' ability.
pub fn is_one_time_witness(module: &CompiledModule, tok: &SignatureToken) -> bool {
    // 1. Argument must be a struct
    let struct_idx = match tok {
        SignatureToken::Datatype(idx) => idx,
        _ => return false,
    };

    let handle = module.datatype_handle_at(*struct_idx);

    // 2. Name must match uppercase module name
    let module_handle = module.module_handle_at(handle.module);
    let module_name = module.identifier_at(module_handle.name).as_str();
    let struct_name = module.identifier_at(handle.name).as_str();
    if struct_name != module_name.to_ascii_uppercase() {
        return false;
    }

    // 3. Must have only the Drop ability
    if handle.abilities != (AbilitySet::EMPTY | Ability::Drop) {
        return false;
    }

    // 4. Must have no type parameters
    if !handle.type_parameters.is_empty() {
        return false;
    }

    // 5. Must have 0 or 1 field (and if 1, it must be Bool)
    let struct_def = match module
        .struct_defs
        .iter()
        .find(|def| def.struct_handle == *struct_idx)
    {
        Some(def) => def,
        None => return false,
    };

    let field_count = struct_def.declared_field_count().unwrap_or(0);
    if field_count > 1 {
        return false;
    }

    if field_count == 1 {
        let field = struct_def.field(0).unwrap();
        if field.signature.0 != SignatureToken::Bool {
            return false;
        }
    }

    true
}
