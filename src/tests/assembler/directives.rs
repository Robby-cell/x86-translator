use super::*;

#[test]
fn test_byte_word_space() {
    let code = ".byte 0xAA\n.word 0xBBCC\n.space 2";
    let bytes = assemble(code).unwrap().bytes;
    // 1 byte + 2 bytes + 2 bytes = 5 bytes total
    assert_eq!(bytes, [0xAA, 0xCC, 0xBB, 0x00, 0x00]);
}

#[test]
fn test_strings() {
    // .ascii "ABC"
    let bytes = assemble(r#".ascii "ABC""#).unwrap().bytes;
    assert_eq!(bytes, [0x41, 0x42, 0x43]);

    // .asciz "AB"
    let bytes = assemble(r#".asciz "AB""#).unwrap().bytes;
    assert_eq!(bytes, [0x41, 0x42, 0x00]); // Null terminated
}

#[test]
fn test_align_is_noop() {
    let code = ".align 4\nmov eax, 1";
    let bytes = assemble(code).unwrap().bytes;
    // Since iced-x86 abstracts alignments through multi-pass, our tool makes it a no-op
    assert_eq!(bytes.len(), 5);
}
