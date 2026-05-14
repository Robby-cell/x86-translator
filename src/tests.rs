#[cfg(feature = "encoder")]
mod assembler;

#[cfg(feature = "encoder")]
mod errors;

#[cfg(feature = "disassembler")]
mod disassembler;

#[cfg(all(feature = "encoder", feature = "disassembler"))]
mod integration;
