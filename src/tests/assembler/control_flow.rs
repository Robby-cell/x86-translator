use super::*;

#[test]
fn test_local_branch_forward() {
    let code = "jmp target\nnop\ntarget: mov eax, 1";
    let bytes = assemble(code).unwrap().bytes;
    assert_eq!(bytes, [0xEB, 0x01, 0x90, 0xB8, 0x01, 0x00, 0x00, 0x00]);
}

#[test]
fn test_local_branch_backward() {
    let code = "start: nop\njmp start";
    let bytes = assemble(code).unwrap().bytes;
    assert_eq!(bytes, [0x90, 0xEB, 0xFD]);
}

#[test]
fn test_call() {
    let code = "call func\nfunc: ret";
    let bytes = assemble(code).unwrap().bytes;
    assert_eq!(bytes, [0xE8, 0x00, 0x00, 0x00, 0x00, 0xC3]);
}

#[test]
fn test_loop() {
    let code = "start: nop\nloop start";
    let bytes = assemble(code).unwrap().bytes;
    // 90 = NOP, E2 FD = LOOP -3
    assert_eq!(bytes, [0x90, 0xE2, 0xFD]);
}

#[test]
fn test_jrcxz() {
    let code = "start: nop\njrcxz start";
    let bytes = assemble_64_bit(code).unwrap().bytes;
    // 90 = NOP, E3 FD = JRCXZ -3
    assert_eq!(bytes, [0x90, 0xE3, 0xFD]);
}
