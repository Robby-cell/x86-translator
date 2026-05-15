use crate::tests::{assembler::assemble_64_bit, disassembler::disassemble_64_bit};

#[test]
fn test_roundtrip_data_processing() {
    let code = "mov eax, 1\nadd rax, rbx\ncmp rcx, rdx\nret";
    let bytes = assemble_64_bit(code).unwrap().bytes;
    let insts = disassemble_64_bit(&bytes).unwrap();

    assert_eq!(insts[0], "mov     eax,1");
    assert_eq!(insts[1], "add     rax,rbx");
    assert_eq!(insts[2], "cmp     rcx,rdx");
    assert_eq!(insts[3], "ret");
}

#[test]
fn test_roundtrip_memory_and_stack() {
    let code = "push rax\nlea rax, [rbx+0x10]\npop rax";
    let bytes = assemble_64_bit(code).unwrap().bytes;
    let insts = disassemble_64_bit(&bytes).unwrap();

    assert_eq!(insts[0], "push    rax");
    assert_eq!(insts[1], "lea     rax,[rbx+10h]");
    assert_eq!(insts[2], "pop     rax");
}

#[test]
fn test_real_loop_example() {
    let source = r#"
            mov ecx, 0x5
            loop_start:
                inc rax
                dec ecx
                cmp ecx, 0
                jmp loop_start
            "#;
    let bytes = assemble_64_bit(source).unwrap().bytes;
    let insts = disassemble_64_bit(&bytes).unwrap();

    assert_eq!(insts.len(), 5);
    assert_eq!(insts[0], "mov     ecx,5");
    assert_eq!(insts[1], "inc     rax");
    assert_eq!(insts[2], "dec     ecx");
    assert_eq!(insts[3], "cmp     ecx,0");

    // iced defaults to short mappings
    assert_eq!(insts[4], "jmp     short 5");
}
