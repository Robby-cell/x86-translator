use super::*;

#[test]
fn test_nop_ret() {
    let bytes = assemble("nop\nret").unwrap();
    assert_eq!(bytes, vec![0x90, 0xC3]);
}
