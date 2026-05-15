use crate::encoder::translate_instruction;
use crate::error::AsmError;
use crate::parser::parse_statement;
use crate::resolver::{NoSymbolResolver, SymbolResolver};
use crate::types::AssembleResult;

use iced_x86::BlockEncoderOptions;
use iced_x86::code_asm::{CodeAssembler, CodeLabel};
use std::collections::HashMap;

pub struct Assembler<'a> {
    bitness: u32,
    start_address: u64,
    resolver: Option<&'a mut dyn SymbolResolver>,
}

impl<'a> Assembler<'a> {
    pub fn new() -> Self {
        Self {
            bitness: 32, // Default to 32-bit
            start_address: 0,
            resolver: None,
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

    pub fn with_resolver(mut self, resolver: &'a mut dyn SymbolResolver) -> Self {
        self.resolver = Some(resolver);
        self
    }

    pub fn assemble(self, source: &str) -> Result<AssembleResult, AsmError> {
        let mut statements = Vec::new();

        for (line_idx, raw_line) in source.lines().enumerate() {
            let line_num = line_idx + 1;
            let code_part = raw_line
                .split(';')
                .next()
                .unwrap_or("")
                .split('#')
                .next()
                .unwrap_or("")
                .trim();

            if code_part.is_empty() {
                continue;
            }
            statements.push(parse_statement(code_part, line_num)?);
        }

        let mut asm = CodeAssembler::new(self.bitness).unwrap();
        let mut code_labels: HashMap<String, CodeLabel> = HashMap::new();

        let mut dummy_resolver = NoSymbolResolver;
        let resolver = self.resolver.unwrap_or(&mut dummy_resolver);

        for stmt in &statements {
            if let Some(lbl_name) = &stmt.label {
                let mut code_label = *code_labels
                    .entry(lbl_name.clone())
                    .or_insert_with(|| asm.create_label());
                asm.set_label(&mut code_label)
                    .map_err(|e| AsmError::EncodeError {
                        line: stmt.line,
                        message: e.to_string(),
                    })?;
                code_labels.insert(lbl_name.clone(), code_label);
            }

            translate_instruction(&mut asm, stmt, &mut code_labels, resolver)?;
        }

        let instruction_count = asm.instructions().len();
        let result = asm.assemble_options(
            self.start_address,
            BlockEncoderOptions::RETURN_NEW_INSTRUCTION_OFFSETS,
        );

        match result {
            Ok(asm_result) => {
                let mut exported_labels = HashMap::new();
                for (name, label) in code_labels {
                    if let Ok(ip) = asm_result.label_ip(&label) {
                        exported_labels.insert(name, ip);
                    }
                }

                Ok(AssembleResult {
                    bytes: asm_result.inner.code_buffer,
                    entry_point: self.start_address,
                    labels: exported_labels,
                    instruction_count,
                })
            }
            Err(e) => Err(AsmError::EncodeError {
                line: 0,
                message: format!("Link/Assemble Failed: {}", e),
            }),
        }
    }
}

/// Helper function for quick, default 32-bit assembly
pub fn assemble(source: &str) -> Result<AssembleResult, AsmError> {
    Assembler::new().assemble(source)
}
