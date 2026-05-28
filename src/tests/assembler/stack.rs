use super::*;

#[test]
fn test_push_pop_register() {
    let bytes = assemble_64_bit("push rax\npop rbx").unwrap().bytes;
    assert_eq!(bytes, [0x50, 0x5B]);
}

#[test]
fn test_push_immediate() {
    let bytes = assemble("push 0x10").unwrap().bytes;
    assert_eq!(bytes, [0x6A, 0x10]);
}
