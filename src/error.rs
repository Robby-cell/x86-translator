use core::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum DecodeError {
    UnalignedInput,
    UnknownInstruction { offset: usize },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::UnalignedInput => write!(f, "Input byte slice unaligned or malformed"),
            DecodeError::UnknownInstruction { offset } => {
                write!(f, "Unknown instruction at offset 0x{:X}", offset)
            }
        }
    }
}
impl core::error::Error for DecodeError {}

#[derive(Debug, Clone, PartialEq)]
pub enum AsmError {
    ParseError {
        line: usize,
        col: usize,
        message: String,
    },
    UndefinedLabel {
        line: usize,
        label: String,
    },
    UnknownMnemonic {
        line: usize,
        mnemonic: String,
    },
    EncodeError {
        line: usize,
        message: String,
    },
}

impl fmt::Display for AsmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AsmError::ParseError { line, col, message } => {
                write!(f, "line {}:{}: {}", line, col, message)
            }
            AsmError::UndefinedLabel { line, label } => {
                write!(f, "line {}: undefined label '{}'", line, label)
            }
            AsmError::UnknownMnemonic { line, mnemonic } => {
                write!(f, "line {}: unknown mnemonic '{}'", line, mnemonic)
            }
            AsmError::EncodeError { line, message } => {
                write!(f, "line {}: encode error: {}", line, message)
            }
        }
    }
}
impl core::error::Error for AsmError {}
