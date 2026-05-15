use super::*;

#[test]
fn test_mov_register() {
    let bytes = assemble("mov eax, ebx").unwrap().bytes;
    assert_eq!(bytes, vec![0x89, 0xD8]);

    let bytes = assemble("mov rax, rbx").unwrap().bytes;
    assert_eq!(bytes, vec![0x48, 0x89, 0xD8]);
}

#[test]
fn test_mov_immediate() {
    let bytes = assemble("mov eax, 1").unwrap().bytes;
    assert_eq!(bytes, vec![0xB8, 0x01, 0x00, 0x00, 0x00]);

    let bytes = assemble("mov rax, 0x1234").unwrap().bytes;
    assert_eq!(
        bytes,
        vec![0x48, 0xB8, 0x34, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
}

#[test]
fn test_add_sub() {
    let bytes = assemble("add rax, rbx").unwrap().bytes;
    assert_eq!(bytes, vec![0x48, 0x01, 0xD8]);

    // SUB EAX, imm32 encoding by iced-x86 is 2D id
    let bytes = assemble("sub eax, 0x10").unwrap().bytes;
    assert_eq!(bytes, vec![0x2D, 0x10, 0x00, 0x00, 0x00]);
}

#[test]
fn test_cmp() {
    let bytes = assemble("cmp rax, rbx").unwrap().bytes;
    assert_eq!(bytes, vec![0x48, 0x39, 0xD8]);
}

#[test]
fn test_inc_dec() {
    let bytes = assemble("inc rax\ndec rbx").unwrap().bytes;
    assert_eq!(bytes, vec![0x48, 0xFF, 0xC0, 0x48, 0xFF, 0xCB]);
}
