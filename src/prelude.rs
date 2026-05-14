pub use crate::error::{AsmError, DecodeError};
pub use crate::options::{AssemblerOptions, DisassemblerOptions, Endian};
pub use crate::resolver::{FnSymbolResolver, HashMapSymbolResolver, SymbolResolver};

#[cfg(feature = "encoder")]
pub use crate::assembler::{Encoder, assemble, assemble_with_options};

#[cfg(feature = "disassembler")]
pub use crate::disassembler::{Decoder, disassemble, disassemble_with_options};
