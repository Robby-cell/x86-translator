use x86_translator::{assembler::Encoder, disassembler::Decoder, options::AssemblerOptions};

fn main() {
    let mut enc = Encoder::new(AssemblerOptions::default());
    let c = enc
        .assemble(
            r#"
    mov eax, 16
    nop
    add eax, ebx"#,
        )
        .unwrap();

    dbg!(&c);

    let dec = Decoder::new(Default::default());
    let o = dec.disassemble(&c).unwrap();

    let _ = dbg!(o);
}
