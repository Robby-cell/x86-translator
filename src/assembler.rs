use crate::encoder::translate_instruction;
use crate::error::AsmError;
use crate::options::AssemblerOptions;
use crate::parser::parse_statement;
use iced_x86::code_asm::{CodeAssembler, CodeLabel};
use std::collections::HashMap;

pub struct Encoder {
    pub options: AssemblerOptions,
}

impl Encoder {
    pub fn new(options: AssemblerOptions) -> Self {
        Self { options }
    }

    pub fn assemble(&mut self, source: &str) -> Result<Vec<u8>, AsmError> {
        let mut statements = Vec::new();

        for (line_idx, raw_line) in source.lines().enumerate() {
            let line_num = line_idx + 1;
            let trimmed = raw_line
                .split(';')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty());
            for part in trimmed {
                let stmt = parse_statement(part, line_num)?;
                statements.push(stmt);
            }
        }

        let mut asm = CodeAssembler::new(self.options.bitness).unwrap();
        let mut labels: HashMap<String, CodeLabel> = HashMap::new();

        for stmt in &statements {
            if let Some(lbl_name) = &stmt.label {
                let mut code_label = *labels
                    .entry(lbl_name.clone())
                    .or_insert_with(|| asm.create_label());
                asm.set_label(&mut code_label)
                    .map_err(|e| AsmError::EncodeError {
                        line: stmt.line,
                        message: e.to_string(),
                    })?;
                labels.insert(lbl_name.clone(), code_label);
            }

            translate_instruction(
                &mut asm,
                stmt,
                &mut labels,
                &mut *self.options.symbol_resolver,
            )?;
        }

        match asm.assemble(self.options.start_address) {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(AsmError::EncodeError {
                line: 0,
                message: format!("Link/Assemble Failed: {}", e),
            }),
        }
    }
}

pub fn assemble(source: &str) -> Result<Vec<u8>, AsmError> {
    Encoder::new(AssemblerOptions::default()).assemble(source)
}

pub fn assemble_with_options(source: &str, options: AssemblerOptions) -> Result<Vec<u8>, AsmError> {
    Encoder::new(options).assemble(source)
}
