use crate::prelude::assemble;

#[test]
fn test_invalid_mnemonic() {
    let res = assemble("fake_instruction eax");
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("unknown mnemonic"));
}

#[test]
fn test_invalid_register() {
    // Because fake_reg defaults parsed as an Operand::Label, it triggers the operand pairing error in encoder
    let res = assemble("mov fake_reg, 1");
    assert!(res.is_err());
    assert!(
        res.unwrap_err()
            .to_string()
            .contains("Unsupported operand pairing")
    );
}

#[test]
fn test_size_mismatch() {
    let res = assemble("mov rax, ebx");
    assert!(res.is_err());
    assert!(
        res.unwrap_err()
            .to_string()
            .contains("Register size mismatch")
    );
}
