use crate::error::DecodeError;

use iced_x86::{Decoder as IcedDecoder, DecoderOptions, Formatter, IntelFormatter};

pub struct Disassembler {
    pub bitness: u32,
    pub start_address: u64,
}

impl Disassembler {
    pub fn new() -> Self {
        Self {
            bitness: 32,
            start_address: 0,
        }
    }

    pub fn bitness(mut self, bitness: u32) -> Self {
        self.bitness = bitness;
        self
    }

    pub fn start_address(mut self, addr: u64) -> Self {
        self.start_address = addr;
        self
    }

    pub fn disassemble(&self, bytes: &[u8]) -> Result<Vec<String>, DecodeError> {
        let mut decoder = IcedDecoder::with_ip(
            self.bitness,
            bytes,
            self.start_address,
            DecoderOptions::NONE,
        );

        let mut formatter = IntelFormatter::new();
        formatter.options_mut().set_first_operand_char_index(8);

        let mut results = Vec::new();
        for instruction in &mut decoder {
            if instruction.is_invalid() {
                return Err(DecodeError::UnknownInstruction {
                    offset: (instruction.ip() - self.start_address) as usize,
                });
            }

            let mut output = String::new();
            formatter.format(&instruction, &mut output);
            results.push(output);
        }

        Ok(results)
    }
}

/// Helper for quick, default 32-bit disassembly
pub fn disassemble(bytes: &[u8]) -> Result<Vec<String>, DecodeError> {
    Disassembler::new().disassemble(bytes)
}
