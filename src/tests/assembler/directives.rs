use super::*;

#[test]
fn test_byte_word_space() {
    let code = ".byte 0xAA\n.word 0xBBCC\n.space 2";
    let bytes = assemble(code).unwrap();
    // 1 byte + 2 bytes + 2 bytes = 5 bytes total
    assert_eq!(bytes, vec![0xAA, 0xCC, 0xBB, 0x00, 0x00]);
}
