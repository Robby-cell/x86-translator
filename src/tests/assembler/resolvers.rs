use super::*;

#[test]
fn test_hashmap_resolver() {
    let mut resolver = symbols!(("external_jmp", 0x1000));

    let bytes = Assembler::new()
        .with_resolver(&mut resolver)
        .assemble("jmp external_jmp")
        .unwrap()
        .bytes;
    assert_eq!(bytes, vec![0xE9, 0xFB, 0x0F, 0x00, 0x00]);
}

#[test]
fn test_fn_resolver_mutability() {
    let mut calls = 0;
    let mut resolver = FnSymbolResolver::new(move |name: &str| {
        calls += 1;
        let _ = calls;
        if name == "printf" { Some(0x2000) } else { None }
    });
    let _ = calls;

    let bytes = Assembler::new()
        .with_resolver(&mut resolver)
        .assemble("call printf")
        .unwrap()
        .bytes;
    assert_eq!(bytes[0], 0xE8);
}
