use crate::prelude::{DisassemblerOptions, disassemble, disassemble_with_options};

#[test]
fn test_disassemble_basic() {
    let bytes = [0xB8, 0x01, 0x00, 0x00, 0x00, 0x50, 0x90, 0xC3];
    let insts = disassemble(&bytes).unwrap();

    assert_eq!(insts.len(), 4);
    assert_eq!(insts[0], "mov     eax,1");
    assert_eq!(insts[1], "push    rax");
    assert_eq!(insts[2], "nop");
    assert_eq!(insts[3], "ret");
}

#[test]
fn test_disassemble_memory() {
    let bytes = [0x48, 0x8D, 0x43, 0x20];
    let insts = disassemble(&bytes).unwrap();

    assert_eq!(insts.len(), 1);
    assert_eq!(insts[0], "lea     rax,[rbx+20h]");
}

#[test]
fn test_disassemble_offset() {
    let options = DisassemblerOptions {
        start_address: 0x1000,
        ..DisassemblerOptions::default()
    };

    let bytes = [0xE8, 0x00, 0x00, 0x00, 0x00];
    let insts = disassemble_with_options(&bytes, options).unwrap();

    // Expected padded offset string mapping from iced
    assert_eq!(insts[0], "call    0000000000001005h");
}

#[test]
fn test_invalid_instruction() {
    let bytes = [0xFF, 0xFF, 0xFF, 0xFF];
    let res = disassemble(&bytes);
    assert!(res.is_err());
}
