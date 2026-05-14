use super::*;

#[test]
fn test_hashmap_resolver() {
    let resolver = symbols!(("external_jmp", 0x1000));
    let options = AssemblerOptions {
        symbol_resolver: Box::new(resolver),
        ..AssemblerOptions::default()
    };

    let bytes = assemble_with_options("jmp external_jmp", options).unwrap();
    assert_eq!(bytes, vec![0xE9, 0xFB, 0x0F, 0x00, 0x00]);
}

#[test]
fn test_fn_resolver_mutability() {
    let mut calls = 0;
    let resolver = FnSymbolResolver::new(move |name: &str| {
        calls += 1;
        let _ = calls;
        if name == "printf" { Some(0x2000) } else { None }
    });
    let _ = calls;

    let options = AssemblerOptions {
        symbol_resolver: Box::new(resolver),
        ..AssemblerOptions::default()
    };

    let bytes = assemble_with_options("call printf", options).unwrap();
    assert_eq!(bytes[0], 0xE8);
}
