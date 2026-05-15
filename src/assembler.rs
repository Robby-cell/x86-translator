use crate::encoder::translate_instruction;
use crate::error::AsmError;
use crate::options::AssemblerOptions;
use crate::parser::parse_statement;
use crate::types::AssembleResult;
use iced_x86::BlockEncoderOptions;
use iced_x86::code_asm::{CodeAssembler, CodeLabel};
use std::collections::HashMap;

pub struct Encoder {
    pub options: AssemblerOptions,
}

impl Encoder {
    pub fn new(options: AssemblerOptions) -> Self {
        Self { options }
    }

    pub fn assemble(&mut self, source: &str) -> Result<AssembleResult, AsmError> {
        let mut statements = Vec::new();

        for (line_idx, raw_line) in source.lines().enumerate() {
            let line_num = line_idx + 1;

            // Strictly strip out comments before processing
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

            let stmt = parse_statement(code_part, line_num)?;
            statements.push(stmt);
        }

        let mut asm = CodeAssembler::new(self.options.bitness).unwrap();
        let mut code_labels: HashMap<String, CodeLabel> = HashMap::new();

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

            translate_instruction(
                &mut asm,
                stmt,
                &mut code_labels,
                &mut *self.options.symbol_resolver,
            )?;
        }

        let instruction_count = asm.instructions().len();

        let result = asm.assemble_options(
            self.options.start_address,
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
                    entry_point: self.options.start_address,
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

pub fn assemble(source: &str) -> Result<AssembleResult, AsmError> {
    Encoder::new(AssemblerOptions::default()).assemble(source)
}

pub fn assemble_with_options(
    source: &str,
    options: AssemblerOptions,
) -> Result<AssembleResult, AsmError> {
    Encoder::new(options).assemble(source)
}
