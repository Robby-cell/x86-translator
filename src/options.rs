#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endian {
    Big = 1 << 30,
    Little = 0,
}
