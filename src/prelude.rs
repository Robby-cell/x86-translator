pub use crate::error::{AsmError, DecodeError};
pub use crate::resolver::{FnSymbolResolver, HashMapSymbolResolver, SymbolResolver};
pub use crate::types::AssembleResult;

#[cfg(feature = "encoder")]
pub use crate::assembler::{Assembler, assemble};

#[cfg(feature = "disassembler")]
pub use crate::disassembler::{Disassembler, disassemble};
