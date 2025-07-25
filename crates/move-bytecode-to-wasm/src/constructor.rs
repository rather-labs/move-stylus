use move_binary_format::file_format::{CompiledModule, SignatureToken, Visibility, FunctionDefinitionIndex, FunctionDefinition, Signature};
use crate::translation::table::FunctionId;

const INIT_FUNCTION_NAME: &str = "init";
// The init() function is a special function that is called once when the module is first deployed,
// so it is a good place to put the code that initializes module's objects and sets up the environment and configuration.
//
// For the init() function to be considered valid, it must adhere to the following requirements:
// 1. It must be named `init`.
// 2. It must be private.
// 3. It must have TxContext as its last argument, with an optional One Time Witness (OTW) as its first argument.
// 4. It must not return any values.
//
// fun init(ctx: &TxContext) { /* ... */}
// fun init(otw: OTW, ctx: &mut TxContext) { /* ... */ }


/// Check if the given function (by index) is a valid `init` function.
pub fn is_init(function_id: FunctionId, move_function_arguments: &Signature, move_function_return: &Signature, function_def: &FunctionDefinition) -> bool {
    // 1. Must be named `init`
    if function_id.identifier != INIT_FUNCTION_NAME {
        return false;
    }

    // 2. Must be private
    if function_def.visibility != Visibility::Private {
        panic!("init() functions must be private");
    }

    // 3. Must have TxContext as last argument (with optional OTW as first argument)
    if move_function_arguments.is_empty() {
        panic!("init() functions must have at least one argument");
    }

    if move_function_arguments.len() > 2 {
        panic!("init() functions must have at most two arguments");
    }

    if !move_function_return.is_empty() {
        panic!("init() functions must return no values");
    }

    // // if !is_tx_context(&params[params.len() - 1], move_module) {
    // //     return false;
    // // }

    // if params.len() == 2 && !is_otw(&params[0], move_module) {
    //     return false;
    // }

    true
}

// /// Check if a SignatureToken corresponds to TxContext
// fn is_tx_context(token: &SignatureToken, move_module: &CompiledModule) -> bool {
//     match token {
//         SignatureToken::Reference(inner) | SignatureToken::MutableReference(inner) => {
//             is_tx_context(inner, move_module)
//         }
//         SignatureToken::Struct(idx) | SignatureToken::StructInstantiation(idx, _) => {
//             let handle = move_module.struct_handle_at(*idx);
//             move_module.identifier_at(handle.name).as_str() == "TxContext"
//         }
//         _ => false,
//     }
// }

// /// Check if a SignatureToken corresponds to OTW (One Time Witness)
// fn is_otw(token: &SignatureToken, move_module: &CompiledModule) -> bool {
//     match token {
//         SignatureToken::Struct(idx) | SignatureToken::StructInstantiation(idx, _) => {
//             let handle = move_module.struct_handle_at(*idx);
//             let name = move_module.identifier_at(handle.name).as_str();
//             name == "ONE_TIME" || name.ends_with("OTW")
//         }
//         _ => false,
//     }
// }
