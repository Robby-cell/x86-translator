use super::*;

#[test]
fn test_nop_ret() {
    let bytes = assemble("nop\nret").unwrap().bytes;
    assert_eq!(bytes, [0x90, 0xC3]);
}
