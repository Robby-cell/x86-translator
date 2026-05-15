use crate::{disassembler::Disassembler, error::DecodeError, prelude::disassemble};

pub(crate) fn disassemble_64_bit(bytes: &[u8]) -> Result<Vec<String>, DecodeError> {
    Disassembler::new().bitness(64).disassemble(bytes)
}

#[test]
fn test_disassemble_basic() {
    let bytes = [0xB8, 0x01, 0x00, 0x00, 0x00, 0x50, 0x90, 0xC3];
    let insts = disassemble_64_bit(&bytes).unwrap();

    assert_eq!(insts.len(), 4);
    assert_eq!(insts[0], "mov     eax,1");
    assert_eq!(insts[1], "push    rax");
    assert_eq!(insts[2], "nop");
    assert_eq!(insts[3], "ret");
}

#[test]
fn test_disassemble_memory() {
    let bytes = [0x48, 0x8D, 0x43, 0x20];
    let insts = disassemble_64_bit(&bytes).unwrap();

    assert_eq!(insts.len(), 1);
    assert_eq!(insts[0], "lea     rax,[rbx+20h]");
}

#[test]
fn test_disassemble_offset() {
    let bytes = [0xE8, 0x00, 0x00, 0x00, 0x00];
    let insts = Disassembler::new()
        .start_address(0x1000)
        .disassemble(&bytes)
        .unwrap();

    // Expected padded offset string mapping from iced
    assert_eq!(insts[0], "call    00001005h");
}

#[test]
fn test_invalid_instruction() {
    let bytes = [0xFF, 0xFF, 0xFF, 0xFF];
    let res = disassemble(&bytes);
    assert!(res.is_err());
}
