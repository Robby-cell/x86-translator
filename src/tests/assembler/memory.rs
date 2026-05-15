use super::*;

#[test]
fn test_mov_memory_base_only() {
    let bytes = assemble_64_bit("mov rax, [rbx]").unwrap().bytes;
    assert_eq!(bytes, vec![0x48, 0x8B, 0x03]);
}

#[test]
fn test_mov_memory_displacement() {
    let bytes = assemble_64_bit("mov rax, [rbx + 0x10]").unwrap().bytes;
    assert_eq!(bytes, vec![0x48, 0x8B, 0x43, 0x10]);

    let bytes = assemble_64_bit("mov [rcx - 0x08], rdx").unwrap().bytes;
    assert_eq!(bytes, vec![0x48, 0x89, 0x51, 0xF8]);
}

#[test]
fn test_lea() {
    let bytes = assemble_64_bit("lea rax, [rbx + 0x20]").unwrap().bytes;
    assert_eq!(bytes, vec![0x48, 0x8D, 0x43, 0x20]);
}
