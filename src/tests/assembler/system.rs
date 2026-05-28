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

    // syscall
    let bytes = assemble_64_bit("syscall").unwrap().bytes;
    assert_eq!(bytes, vec![0x0F, 0x05]);
}

#[test]
fn test_string_io() {
    let bytes = assemble("cld\nlodsb\noutsb").unwrap().bytes;
    assert_eq!(bytes, vec![0xFC, 0xAC, 0x6E]);
}

#[test]
fn test_in_out() {
    // out dx, al
    let bytes = assemble("out dx, al").unwrap().bytes;
    assert_eq!(bytes, vec![0xEE]);

    // out 0x80, al
    let bytes = assemble("out 0x80, al").unwrap().bytes;
    assert_eq!(bytes, vec![0xE6, 0x80]);

    // in al, dx
    let bytes = assemble("in al, dx").unwrap().bytes;
    assert_eq!(bytes, vec![0xEC]);

    // inb al, 0x80
    let bytes = assemble("inb al, 0x80").unwrap().bytes;
    assert_eq!(bytes, vec![0xE4, 0x80]);
}
