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

    pub fn assemble(mut self, source: &str) -> Result<AssembleResult, AsmError> {
        let mut statements = Vec::new();

        #[cfg(feature = "mapping")]
        let mut label_to_line = HashMap::new();

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
            let stmt = parse_statement(code_part, line_num)?;

            #[cfg(feature = "mapping")]
            if let Some(lbl) = &stmt.label {
                label_to_line.insert(lbl.clone(), line_num);
            }
            statements.push(stmt);
        }

        let mut estimated_labels: HashMap<String, i64> = HashMap::new();
        for stmt in &statements {
            if let Some(lbl) = &stmt.label {
                estimated_labels.insert(lbl.clone(), 0);
            }
        }

        let mut final_asm_result = None;
        let mut final_code_labels = HashMap::new();
        let mut instruction_count = 0;
        let mut last_error = None;

        #[cfg(feature = "mapping")]
        let mut final_block_to_line = Vec::new();

        for _ in 0..5 {
            let mut asm = CodeAssembler::new(self.bitness).unwrap();
            let mut code_labels: HashMap<String, CodeLabel> = HashMap::new();
            let mut pass_failed = false;

            #[cfg(feature = "mapping")]
            let mut block_to_line = Vec::new();

            for stmt in &statements {
                #[cfg(feature = "mapping")]
                let start_blocks = asm.instructions().len();

                if let Some(lbl_name) = &stmt.label {
                    let mut code_label = *code_labels
                        .entry(lbl_name.clone())
                        .or_insert_with(|| asm.create_label());
                    if let Err(e) = asm.set_label(&mut code_label) {
                        last_error = Some(AsmError::EncodeError {
                            line: stmt.line,
                            message: e.to_string(),
                        });
                        pass_failed = true;
                        break;
                    }
                    code_labels.insert(lbl_name.clone(), code_label);
                }

                let mut dummy_resolver = NoSymbolResolver;
                let resolver: &mut dyn SymbolResolver = match &mut self.resolver {
                    Some(r) => &mut **r,
                    None => &mut dummy_resolver,
                };

                if let Err(e) = translate_instruction(
                    &mut asm,
                    stmt,
                    &mut code_labels,
                    &estimated_labels,
                    resolver,
                    self.start_address,
                ) {
                    last_error = Some(e);
                    pass_failed = true;
                    break;
                }

                #[cfg(feature = "mapping")]
                {
                    let end_blocks = asm.instructions().len();
                    for _ in start_blocks..end_blocks {
                        block_to_line.push(stmt.line);
                    }
                }
            }

            if pass_failed {
                break;
            }

            instruction_count = asm.instructions().len();
            match asm.assemble_options(
                self.start_address,
                BlockEncoderOptions::RETURN_NEW_INSTRUCTION_OFFSETS,
            ) {
                Ok(asm_result) => {
                    let mut changed = false;
                    for (name, label) in &code_labels {
                        if let Ok(ip) = asm_result.label_ip(label) {
                            let offset = (ip - self.start_address) as i64;
                            if estimated_labels.get(name) != Some(&offset) {
                                estimated_labels.insert(name.clone(), offset);
                                changed = true;
                            }
                        }
                    }

                    final_asm_result = Some(asm_result);
                    final_code_labels = code_labels;
                    #[cfg(feature = "mapping")]
                    {
                        final_block_to_line = block_to_line;
                    }

                    if !changed {
                        break;
                    }
                }
                Err(e) => {
                    last_error = Some(AsmError::EncodeError {
                        line: 0,
                        message: format!("Link/Assemble Failed: {}", e),
                    });
                    break;
                }
            }
        }

        match final_asm_result {
            Some(asm_result) => {
                let mut exported_labels = HashMap::new();
                for (name, label) in final_code_labels {
                    if let Ok(ip) = asm_result.label_ip(&label) {
                        exported_labels.insert(name, ip);
                    }
                }

                #[cfg(feature = "mapping")]
                let mut ip_to_line = HashMap::new();
                #[cfg(feature = "mapping")]
                let mut line_to_ip = HashMap::new();

                #[cfg(feature = "mapping")]
                {
                    for (block_idx, &offset) in
                        asm_result.inner.new_instruction_offsets.iter().enumerate()
                    {
                        if let Some(&line) = final_block_to_line.get(block_idx) {
                            let ip = self.start_address + offset as u64;
                            ip_to_line.insert(ip, line);
                            line_to_ip.entry(line).or_insert(ip);
                        }
                    }
                }

                Ok(AssembleResult {
                    bytes: asm_result.inner.code_buffer,
                    entry_point: self.start_address,
                    labels: exported_labels,
                    instruction_count,
                    #[cfg(feature = "mapping")]
                    ip_to_line,
                    #[cfg(feature = "mapping")]
                    line_to_ip,
                    #[cfg(feature = "mapping")]
                    label_to_line,
                })
            }
            None => Err(last_error.unwrap_or(AsmError::EncodeError {
                line: 0,
                message: "Assembly failed to converge".into(),
            })),
        }
    }
}

/// Helper function for quick, default 32-bit assembly
pub fn assemble(source: &str) -> Result<AssembleResult, AsmError> {
    Assembler::new().assemble(source)
}
