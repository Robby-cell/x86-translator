use super::*;

#[test]
fn test_conditional_jumps() {
    // je target
    let code = "je target\nnop\ntarget: nop";
    let bytes = assemble(code).unwrap().bytes;
    assert_eq!(bytes[0], 0x74); // 74 is short JE
}

#[test]
fn test_cmov() {
    // cmove rax, rbx
    let bytes = assemble_64_bit("cmove rax, rbx").unwrap().bytes;
    assert_eq!(bytes, [0x48, 0x0F, 0x44, 0xC3]);
}

#[test]
fn test_setcc() {
    // sete al
    let bytes = assemble("sete al").unwrap().bytes;
    assert_eq!(bytes, [0x0F, 0x94, 0xC0]);
}
