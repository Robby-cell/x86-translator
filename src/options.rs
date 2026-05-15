use crate::resolver::{NoSymbolResolver, SymbolResolver};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endian {
    Big = 1 << 30,
    Little = 0,
}

pub struct AssemblerOptions {
    pub start_address: u64,
    pub bitness: u32,
    pub endian: Endian,
    pub symbol_resolver: Box<dyn SymbolResolver>,
}

impl Default for AssemblerOptions {
    fn default() -> Self {
        AssemblerOptions {
            start_address: 0,
            bitness: 32,
            endian: Endian::Little,
            symbol_resolver: Box::new(NoSymbolResolver),
        }
    }
}

pub struct DisassemblerOptions {
    pub start_address: u64,
    pub bitness: u32,
    pub endian: Endian,
}

impl Default for DisassemblerOptions {
    fn default() -> Self {
        DisassemblerOptions {
            start_address: 0,
            bitness: 32,
            endian: Endian::Little,
        }
    }
}
