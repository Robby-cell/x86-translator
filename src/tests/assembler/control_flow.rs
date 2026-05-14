use super::*;

#[test]
fn test_local_branch_forward() {
    let code = "jmp target\nnop\ntarget: mov eax, 1";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0xEB, 0x01, 0x90, 0xB8, 0x01, 0x00, 0x00, 0x00]);
}

#[test]
fn test_local_branch_backward() {
    let code = "start: nop\njmp start";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x90, 0xEB, 0xFD]);
}

#[test]
fn test_call() {
    let code = "call func\nfunc: ret";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0xE8, 0x00, 0x00, 0x00, 0x00, 0xC3]);
}
