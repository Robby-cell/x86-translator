use super::*;

#[test]
fn test_system_calls() {
    // hlt
    let bytes = assemble("hlt").unwrap().bytes;
    assert_eq!(bytes, vec![0xF4]);

    // int 0x80 (Linux syscall)
    let bytes = assemble("int 0x80").unwrap().bytes;
    assert_eq!(bytes, vec![0xCD, 0x80]);

    // int 3 (Breakpoint)
    let bytes = assemble("int 3").unwrap().bytes;
    assert_eq!(bytes, vec![0xCD, 0x03]);
}
