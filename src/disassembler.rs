#![cfg(feature = "disassembler")]
use crate::error::DecodeError;
use crate::options::DisassemblerOptions;
use iced_x86::{Decoder as IcedDecoder, DecoderOptions, Formatter, IntelFormatter};

pub struct Decoder {
    pub options: DisassemblerOptions,
}

impl Decoder {
    pub fn new(options: DisassemblerOptions) -> Self {
        Self { options }
    }

    pub fn disassemble(&self, bytes: &[u8]) -> Result<Vec<String>, DecodeError> {
        let mut decoder = IcedDecoder::with_ip(
            self.options.bitness,
            bytes,
            self.options.start_address,
            DecoderOptions::NONE,
        );

        let mut formatter = IntelFormatter::new();
        // Mimic standard objdump padding layout
        formatter.options_mut().set_first_operand_char_index(8);

        let mut results = Vec::new();
        for instruction in &mut decoder {
            if instruction.is_invalid() {
                return Err(DecodeError::UnknownInstruction {
                    offset: (instruction.ip() - self.options.start_address) as usize,
                });
            }

            let mut output = String::new();
            formatter.format(&instruction, &mut output);
            results.push(output);
        }

        Ok(results)
    }
}

/// Standalone helper: Disassemble bytes using default options.
pub fn disassemble(bytes: &[u8]) -> Result<Vec<String>, DecodeError> {
    Decoder::new(DisassemblerOptions::default()).disassemble(bytes)
}

/// Standalone helper: Disassemble bytes using explicit options.
pub fn disassemble_with_options(
    bytes: &[u8],
    options: DisassemblerOptions,
) -> Result<Vec<String>, DecodeError> {
    Decoder::new(options).disassemble(bytes)
}
