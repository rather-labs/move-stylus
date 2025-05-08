module 0x00::use_stdlib;

public fun test_append() {
    let mut str = b"hello".to_ascii_string();
    str.append(b" world".to_ascii_string());

    assert!(str == b"hello world".to_ascii_string());
}
