pub mod error;
pub mod options;
pub mod resolver;
pub mod types;

#[cfg(feature = "encoder")]
pub mod assembler;
#[cfg(feature = "encoder")]
pub mod encoder;
#[cfg(feature = "encoder")]
pub mod parser;

#[cfg(feature = "disassembler")]
pub mod disassembler;

pub mod prelude;

#[cfg(test)]
mod tests;
