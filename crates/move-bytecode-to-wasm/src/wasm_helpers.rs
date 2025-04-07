use walrus::{FunctionId, Module};

use crate::translation::functions::get_function_body_builder;

pub fn load_i32_from_bytes_instructions(
    module: &mut Module,
    function_id: FunctionId,
    bytes: &[u8],
) {
    assert!(bytes.len() <= 4, "Constant is too large to fit in u32");

    // pad to 4 bytes on the right
    let mut bytes = bytes.to_vec();
    bytes.resize(4, 0);

    get_function_body_builder(module, function_id)
        .i32_const(i32::from_le_bytes(bytes.try_into().unwrap()));
}

pub fn load_i64_from_bytes_instructions(
    module: &mut Module,
    function_id: FunctionId,
    bytes: &[u8],
) {
    assert!(bytes.len() <= 8, "Constant is too large to fit in u64");

    // pad to 8 bytes on the right
    let mut bytes = bytes.to_vec();
    bytes.resize(8, 0);

    get_function_body_builder(module, function_id)
        .i64_const(i64::from_le_bytes(bytes.try_into().unwrap()));
}
