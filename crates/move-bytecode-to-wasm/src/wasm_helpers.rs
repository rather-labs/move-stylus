use walrus::InstrSeqBuilder;

pub fn load_i32_from_bytes_instructions(builder: &mut InstrSeqBuilder, bytes: &[u8]) {
    assert!(bytes.len() <= 4, "Constant is too large to fit in u32");

    // pad to 4 bytes on the right
    let mut bytes = bytes.to_vec();
    bytes.resize(4, 0);

    builder.i32_const(i32::from_le_bytes(bytes.try_into().unwrap()));
}

pub fn load_i64_from_bytes_instructions(builder: &mut InstrSeqBuilder, bytes: &[u8]) {
    assert!(bytes.len() <= 8, "Constant is too large to fit in u64");

    // pad to 8 bytes on the right
    let mut bytes = bytes.to_vec();
    bytes.resize(8, 0);

    builder.i64_const(i64::from_le_bytes(bytes.try_into().unwrap()));
}
