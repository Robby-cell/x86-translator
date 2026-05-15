use super::*;

#[test]
fn test_mov_register() {
    let bytes = assemble("mov eax, ebx").unwrap().bytes;
    assert_eq!(bytes, vec![0x89, 0xD8]);

    let bytes = assemble_64_bit("mov rax, rbx").unwrap().bytes;
    assert_eq!(bytes, vec![0x48, 0x89, 0xD8]);
}

#[test]
fn test_mov_immediate() {
    let bytes = assemble("mov eax, 1").unwrap().bytes;
    assert_eq!(bytes, vec![0xB8, 0x01, 0x00, 0x00, 0x00]);

    let bytes = assemble_64_bit("mov rax, 0x1234").unwrap().bytes;
    // Full instruction:
    // 48 b8 34 12 00 00 00 00 00 00
    // First optimization:
    // assert_eq!(bytes, vec![0x48, 0xC7, 0xC0, 0x34, 0x12, 0x00, 0x00]);

    // Optimized to 32-bit `mov eax, 0x1234`
    assert_eq!(bytes, [0xB8, 0x34, 0x12, 0x00, 0x00]);
}

#[test]
fn test_add_sub() {
    let bytes = assemble("add eax, ebx").unwrap().bytes;
    assert_eq!(bytes, vec![0x01, 0xD8]);

    // SUB EAX, imm32 encoding by iced-x86 is 2D id (Accumulator specific opcode)
    let bytes = assemble("sub eax, 0x10").unwrap().bytes;
    assert_eq!(bytes, vec![0x2D, 0x10, 0x00, 0x00, 0x00]);

    // ADD EAX, imm32 encoding by iced-x86 is 05 id (Accumulator specific opcode)
    let bytes = assemble("add eax, 5").unwrap().bytes;
    assert_eq!(bytes, vec![0x05, 0x05, 0x00, 0x00, 0x00]);

    // ADD ECX, imm8 encoding by iced-x86 successfully compresses to 83 id
    let bytes = assemble("add ecx, 5").unwrap().bytes;
    assert_eq!(bytes, vec![0x83, 0xC1, 0x05]);
}

#[test]
fn test_cmp() {
    let bytes = assemble_64_bit("cmp rax, rbx").unwrap().bytes;
    assert_eq!(bytes, vec![0x48, 0x39, 0xD8]);
}

#[test]
fn test_inc_dec() {
    let bytes = assemble_64_bit("inc rax\ndec rbx").unwrap().bytes;
    assert_eq!(bytes, vec![0x48, 0xFF, 0xC0, 0x48, 0xFF, 0xCB]);
}

#[test]
fn test_lea_expr() {
    let bytes = assemble("lea eax, [ebx + ecx]").unwrap().bytes;
    assert_eq!(bytes, vec![0x8D, 0x04, 0x0B]);
}

#[test]
fn test_arithmetic_extended() {
    // adc eax, ebx
    let bytes = assemble("adc eax, ebx").unwrap().bytes;
    assert_eq!(bytes, vec![0x11, 0xD8]);

    // sbb ecx, 1
    let bytes = assemble("sbb ecx, 1").unwrap().bytes;
    assert_eq!(bytes, vec![0x83, 0xD9, 0x01]);

    // mul ebx
    let bytes = assemble("mul ebx").unwrap().bytes;
    assert_eq!(bytes, vec![0xF7, 0xE3]);

    // div rcx
    let bytes = assemble_64_bit("div rcx").unwrap().bytes;
    assert_eq!(bytes, vec![0x48, 0xF7, 0xF1]);
}

#[test]
fn test_bitwise() {
    // and eax, ebx
    let bytes = assemble("and eax, ebx").unwrap().bytes;
    assert_eq!(bytes, vec![0x21, 0xD8]);

    // or ecx, 0x10
    let bytes = assemble("or ecx, 0x10").unwrap().bytes;
    assert_eq!(bytes, vec![0x83, 0xC9, 0x10]);

    // xor eax, eax (Standard way to clear register)
    let bytes = assemble("xor eax, eax").unwrap().bytes;
    assert_eq!(bytes, vec![0x31, 0xC0]);

    // test eax, eax
    let bytes = assemble("test eax, eax").unwrap().bytes;
    assert_eq!(bytes, vec![0x85, 0xC0]);
}
