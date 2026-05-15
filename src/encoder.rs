use crate::error::AsmError;
use crate::resolver::SymbolResolver;
use crate::types::*;
use iced_x86::code_asm::*;
use std::collections::HashMap;

fn to_reg32(r: Reg) -> Option<AsmRegister32> {
    match r {
        Reg::Eax => Some(eax),
        Reg::Ecx => Some(ecx),
        Reg::Edx => Some(edx),
        Reg::Ebx => Some(ebx),
        Reg::Esp => Some(esp),
        Reg::Ebp => Some(ebp),
        Reg::Esi => Some(esi),
        Reg::Edi => Some(edi),
        _ => None,
    }
}

fn to_reg64(r: Reg) -> Option<AsmRegister64> {
    match r {
        Reg::Rax => Some(rax),
        Reg::Rcx => Some(rcx),
        Reg::Rdx => Some(rdx),
        Reg::Rbx => Some(rbx),
        Reg::Rsp => Some(rsp),
        Reg::Rbp => Some(rbp),
        Reg::Rsi => Some(rsi),
        Reg::Rdi => Some(rdi),
        _ => None,
    }
}

/// Helper to get the 32-bit equivalent of a 64-bit register for optimization
fn get_sub_reg32(r: Reg) -> Option<AsmRegister32> {
    match r {
        Reg::Rax | Reg::Eax => Some(eax),
        Reg::Rcx | Reg::Ecx => Some(ecx),
        Reg::Rdx | Reg::Edx => Some(edx),
        Reg::Rbx | Reg::Ebx => Some(ebx),
        Reg::Rsp | Reg::Esp => Some(esp),
        Reg::Rbp | Reg::Ebp => Some(ebp),
        Reg::Rsi | Reg::Esi => Some(esi),
        Reg::Rdi | Reg::Edi => Some(edi),
        _ => None,
    }
}

fn build_mem(
    bitness: u32,
    base: Option<Reg>,
    index: Option<Reg>,
    scale: u32,
    disp: i32,
) -> AsmMemoryOperand {
    if bitness == 64 {
        match (base.and_then(to_reg64), index.and_then(to_reg64)) {
            (Some(b), Some(i)) => qword_ptr(b + i * scale as i32 + disp),
            (Some(b), None) => qword_ptr(b + disp),
            (None, Some(i)) => qword_ptr(i * scale as i32 + disp),
            (None, None) => qword_ptr(disp as i64),
        }
    } else {
        match (base.and_then(to_reg32), index.and_then(to_reg32)) {
            (Some(b), Some(i)) => dword_ptr(b + i * scale as i32 + disp),
            (Some(b), None) => dword_ptr(b + disp),
            (None, Some(i)) => dword_ptr(i * scale as i32 + disp),
            (None, None) => dword_ptr(disp as i64),
        }
    }
}

pub fn translate_instruction(
    asm: &mut CodeAssembler,
    stmt: &Statement,
    labels: &mut HashMap<String, CodeLabel>,
    resolver: &mut dyn SymbolResolver,
) -> Result<(), AsmError> {
    let to_err = |e: IcedError| AsmError::EncodeError {
        line: stmt.line,
        message: e.to_string(),
    };
    let bitness = asm.bitness();

    match stmt.mnemonic {
        Mnemonic::LabelOnly | Mnemonic::Global | Mnemonic::Text | Mnemonic::Data => Ok(()),
        Mnemonic::Byte => {
            if let Some(Operand::Expr(Expr::Number(n))) = stmt.operands.get(0) {
                asm.db(&[*n as u8]).map_err(to_err)?;
            }
            Ok(())
        }
        Mnemonic::Short => {
            if let Some(Operand::Expr(Expr::Number(n))) = stmt.operands.get(0) {
                asm.dw(&[*n as u16]).map_err(to_err)?;
            }
            Ok(())
        }
        Mnemonic::Word => {
            if let Some(Operand::Expr(Expr::Number(n))) = stmt.operands.get(0) {
                asm.dd(&[*n as u32]).map_err(to_err)?;
            }
            Ok(())
        }
        Mnemonic::Space => {
            if let Some(Operand::Expr(Expr::Number(n))) = stmt.operands.get(0) {
                asm.db(&vec![0u8; *n as usize]).map_err(to_err)?;
            }
            Ok(())
        }
        Mnemonic::Nop => asm.nop().map_err(to_err),
        Mnemonic::Ret => asm.ret().map_err(to_err),

        Mnemonic::Jmp | Mnemonic::Call => {
            if let Some(Operand::Label(lbl)) = stmt.operands.get(0) {
                if let Some(addr) = resolver.resolve(lbl) {
                    if stmt.mnemonic == Mnemonic::Jmp {
                        asm.jmp(addr).map_err(to_err)?;
                    } else {
                        asm.call(addr).map_err(to_err)?;
                    }
                } else {
                    let code_label = *labels
                        .entry(lbl.clone())
                        .or_insert_with(|| asm.create_label());
                    if stmt.mnemonic == Mnemonic::Jmp {
                        asm.jmp(code_label).map_err(to_err)?;
                    } else {
                        asm.call(code_label).map_err(to_err)?;
                    }
                }
                Ok(())
            } else {
                Err(AsmError::EncodeError {
                    line: stmt.line,
                    message: "Label required".into(),
                })
            }
        }

        Mnemonic::Inc | Mnemonic::Dec => {
            let r = match &stmt.operands[0] {
                Operand::Reg(r) => r,
                _ => {
                    return Err(AsmError::EncodeError {
                        line: stmt.line,
                        message: "Register required".into(),
                    });
                }
            };
            if let Some(reg) = to_reg32(*r) {
                if stmt.mnemonic == Mnemonic::Inc {
                    asm.inc(reg)
                } else {
                    asm.dec(reg)
                }
                .map_err(to_err)?;
            } else if let Some(reg) = to_reg64(*r) {
                if stmt.mnemonic == Mnemonic::Inc {
                    asm.inc(reg)
                } else {
                    asm.dec(reg)
                }
                .map_err(to_err)?;
            }
            Ok(())
        }

        Mnemonic::Push | Mnemonic::Pop => {
            let is_push = stmt.mnemonic == Mnemonic::Push;
            match &stmt.operands[0] {
                Operand::Reg(r) => {
                    if let Some(reg) = to_reg64(*r) {
                        if is_push { asm.push(reg) } else { asm.pop(reg) }.map_err(to_err)?;
                    } else if let Some(reg) = to_reg32(*r) {
                        if is_push { asm.push(reg) } else { asm.pop(reg) }.map_err(to_err)?;
                    }
                }
                Operand::Imm(i) => {
                    if is_push {
                        asm.push(*i as i32).map_err(to_err)?;
                    } else {
                        return Err(AsmError::EncodeError {
                            line: stmt.line,
                            message: "Cannot pop immediate".into(),
                        });
                    }
                }
                _ => {
                    return Err(AsmError::EncodeError {
                        line: stmt.line,
                        message: "Unsupported operand pairing".into(),
                    });
                }
            }
            Ok(())
        }

        Mnemonic::Lea => {
            if let (
                Operand::Reg(r1),
                Operand::Memory {
                    base,
                    index,
                    scale,
                    disp,
                },
            ) = (&stmt.operands[0], &stmt.operands[1])
            {
                let mem = build_mem(bitness, *base, *index, *scale, *disp);
                if let Some(reg) = to_reg64(*r1) {
                    asm.lea(reg, mem).map_err(to_err)?;
                } else if let Some(reg) = to_reg32(*r1) {
                    asm.lea(reg, mem).map_err(to_err)?;
                } else {
                    return Err(AsmError::EncodeError {
                        line: stmt.line,
                        message: "Register size mismatch".into(),
                    });
                }
                Ok(())
            } else {
                Err(AsmError::EncodeError {
                    line: stmt.line,
                    message: "Unsupported operand pairing".into(),
                })
            }
        }

        Mnemonic::Mov => {
            let (o1, o2) = (&stmt.operands[0], &stmt.operands[1]);
            match (o1, o2) {
                (Operand::Reg(r1), Operand::Reg(r2)) => {
                    if let (Some(reg1), Some(reg2)) = (to_reg32(*r1), to_reg32(*r2)) {
                        asm.mov(reg1, reg2).map_err(to_err)?;
                    } else if let (Some(reg1), Some(reg2)) = (to_reg64(*r1), to_reg64(*r2)) {
                        asm.mov(reg1, reg2).map_err(to_err)?;
                    } else {
                        return Err(AsmError::EncodeError {
                            line: stmt.line,
                            message: "Register size mismatch".into(),
                        });
                    }
                }
                (Operand::Reg(r1), Operand::Imm(i)) => {
                    // OPTIMIZATION: mov r64, imm32 (positive) -> mov r32, imm32 (saves 5 bytes due to zero-extension)
                    if bitness == 64 && to_reg64(*r1).is_some() && *i >= 0 && *i <= u32::MAX as i64
                    {
                        let sub_reg = get_sub_reg32(*r1).unwrap();
                        asm.mov(sub_reg, *i as i32).map_err(to_err)?;
                    } else if let Some(reg) = to_reg64(*r1) {
                        asm.mov(reg, *i as i64).map_err(to_err)?;
                    } else if let Some(reg) = to_reg32(*r1) {
                        asm.mov(reg, *i as i32).map_err(to_err)?;
                    }
                }
                (
                    Operand::Reg(r1),
                    Operand::Memory {
                        base,
                        index,
                        scale,
                        disp,
                    },
                ) => {
                    let mem = build_mem(bitness, *base, *index, *scale, *disp);
                    if let Some(reg) = to_reg64(*r1) {
                        asm.mov(reg, mem).map_err(to_err)?;
                    } else if let Some(reg) = to_reg32(*r1) {
                        asm.mov(reg, mem).map_err(to_err)?;
                    } else {
                        return Err(AsmError::EncodeError {
                            line: stmt.line,
                            message: "Register size mismatch".into(),
                        });
                    }
                }
                (
                    Operand::Memory {
                        base,
                        index,
                        scale,
                        disp,
                    },
                    Operand::Reg(r2),
                ) => {
                    let mem = build_mem(bitness, *base, *index, *scale, *disp);
                    if let Some(reg) = to_reg64(*r2) {
                        asm.mov(mem, reg).map_err(to_err)?;
                    } else if let Some(reg) = to_reg32(*r2) {
                        asm.mov(mem, reg).map_err(to_err)?;
                    } else {
                        return Err(AsmError::EncodeError {
                            line: stmt.line,
                            message: "Register size mismatch".into(),
                        });
                    }
                }
                _ => {
                    return Err(AsmError::EncodeError {
                        line: stmt.line,
                        message: "Unsupported operand pairing".into(),
                    });
                }
            }
            Ok(())
        }

        op @ (Mnemonic::Add | Mnemonic::Sub | Mnemonic::Cmp) => {
            let (o1, o2) = (&stmt.operands[0], &stmt.operands[1]);
            match (o1, o2) {
                (Operand::Reg(r1), Operand::Reg(r2)) => {
                    if let (Some(reg1), Some(reg2)) = (to_reg32(*r1), to_reg32(*r2)) {
                        match op {
                            Mnemonic::Add => asm.add(reg1, reg2),
                            Mnemonic::Sub => asm.sub(reg1, reg2),
                            _ => asm.cmp(reg1, reg2),
                        }
                        .map_err(to_err)?;
                    } else if let (Some(reg1), Some(reg2)) = (to_reg64(*r1), to_reg64(*r2)) {
                        match op {
                            Mnemonic::Add => asm.add(reg1, reg2),
                            Mnemonic::Sub => asm.sub(reg1, reg2),
                            _ => asm.cmp(reg1, reg2),
                        }
                        .map_err(to_err)?;
                    } else {
                        return Err(AsmError::EncodeError {
                            line: stmt.line,
                            message: "Register size mismatch".into(),
                        });
                    }
                }
                (Operand::Reg(r1), Operand::Imm(i)) => {
                    if let Some(reg) = to_reg32(*r1) {
                        match op {
                            Mnemonic::Add => asm.add(reg, *i as i32),
                            Mnemonic::Sub => asm.sub(reg, *i as i32),
                            _ => asm.cmp(reg, *i as i32),
                        }
                        .map_err(to_err)?;
                    } else if let Some(reg) = to_reg64(*r1) {
                        match op {
                            Mnemonic::Add => asm.add(reg, *i as i32),
                            Mnemonic::Sub => asm.sub(reg, *i as i32),
                            _ => asm.cmp(reg, *i as i32),
                        }
                        .map_err(to_err)?;
                    }
                }
                _ => {
                    return Err(AsmError::EncodeError {
                        line: stmt.line,
                        message: "Unsupported operand pairing".into(),
                    });
                }
            }
            Ok(())
        }
        _ => Err(AsmError::UnknownMnemonic {
            line: stmt.line,
            mnemonic: format!("{:?}", stmt.mnemonic),
        }),
    }
}
