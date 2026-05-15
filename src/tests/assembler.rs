use crate::assembler::Assembler;
use crate::error::AsmError;
use crate::prelude::{FnSymbolResolver, assemble};
use crate::symbols;
use crate::types::AssembleResult;

mod basic;
mod control_flow;
mod data_processing;
mod directives;
mod memory;
mod resolvers;
mod stack;

pub(crate) fn assemble_64_bit(code: &str) -> Result<AssembleResult, AsmError> {
    Assembler::new().bitness(64).assemble(code)
}

#[test]
fn test_metadata_output() {
    let code = "
    .global _start
    .byte 0
    .dword 0
    _start: 
        mov eax, 1
        push rax
    my_func:
        ret
    ";

    let result = assemble_64_bit(code).unwrap();

    // 5 total blocks (.byte, .dword, mov, push, ret)
    assert_eq!(result.instruction_count, 5);

    // _start is at physical byte address 5 (because of the 1-byte '.byte 0', and 4-byte '.dword 0' preceding it)
    assert_eq!(result.labels.get("_start"), Some(&5));

    // 'mov eax, 1' takes 5 bytes (0xB8 0x01 0x00 0x00 0x00)
    // 'push rax' takes 1 byte (0x50)
    // my_func is at 5 + 5 + 1 = 11
    assert_eq!(result.labels.get("my_func"), Some(&11)); // Can also use &0x0B

    // Flat binary defaults entry point to the start address (0x0)
    assert_eq!(result.entry_point, 0);
}
