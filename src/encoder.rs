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

fn to_mem64(base: Reg, disp: i32) -> Option<AsmMemoryOperand> {
    Some(qword_ptr(to_reg64(base)? + disp))
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
                let count = *n as usize;
                if count > 0 {
                    asm.db(&vec![0u8; count]).map_err(to_err)?;
                }
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
                    labels.insert(lbl.clone(), code_label);
                }
                Ok(())
            } else {
                Err(AsmError::EncodeError {
                    line: stmt.line,
                    message: "Branch requires label".into(),
                })
            }
        }

        Mnemonic::Inc | Mnemonic::Dec => {
            match &stmt.operands[0] {
                Operand::Reg(r) => {
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
                    } else {
                        return Err(AsmError::EncodeError {
                            line: stmt.line,
                            message: "Invalid inc/dec register".into(),
                        });
                    }
                }
                _ => {
                    return Err(AsmError::EncodeError {
                        line: stmt.line,
                        message: "Unsupported inc/dec operand".into(),
                    });
                }
            }
            Ok(())
        }

        Mnemonic::Push | Mnemonic::Pop => {
            let is_push = stmt.mnemonic == Mnemonic::Push;
            match &stmt.operands[0] {
                Operand::Reg(r) => {
                    if let Some(reg) = to_reg64(*r) {
                        if is_push { asm.push(reg) } else { asm.pop(reg) }.map_err(to_err)?;
                    }
                }
                Operand::Imm(i) => if is_push {
                    asm.push(*i as i32)
                } else {
                    return Err(AsmError::EncodeError {
                        line: stmt.line,
                        message: "Cannot pop immediate".into(),
                    });
                }
                .map_err(to_err)?,
                _ => {
                    return Err(AsmError::EncodeError {
                        line: stmt.line,
                        message: "Unsupported push/pop operand".into(),
                    });
                }
            }
            Ok(())
        }

        Mnemonic::Lea => {
            let o1 = &stmt.operands[0];
            let o2 = &stmt.operands[1];
            if let (Operand::Reg(r1), Operand::Memory { base, disp }) = (o1, o2) {
                if let (Some(reg1), Some(mem)) = (to_reg64(*r1), to_mem64(*base, *disp)) {
                    asm.lea(reg1, mem).map_err(to_err)?;
                } else if let (Some(reg1), Some(mem)) = (to_reg32(*r1), to_mem64(*base, *disp)) {
                    asm.lea(reg1, mem).map_err(to_err)?;
                } else {
                    return Err(AsmError::EncodeError {
                        line: stmt.line,
                        message: "Unsupported LEA sizes".into(),
                    });
                }
            } else {
                return Err(AsmError::EncodeError {
                    line: stmt.line,
                    message: "LEA requires Reg, Mem".into(),
                });
            }
            Ok(())
        }

        op @ (Mnemonic::Mov | Mnemonic::Add | Mnemonic::Sub | Mnemonic::Cmp) => {
            let o1 = &stmt.operands[0];
            let o2 = &stmt.operands[1];

            macro_rules! bin_op {
                ($func:ident, $imm64_cast:ident) => {
                    match (o1, o2) {
                        (Operand::Reg(r1), Operand::Reg(r2)) => {
                            if let (Some(reg1), Some(reg2)) = (to_reg32(*r1), to_reg32(*r2)) {
                                asm.$func(reg1, reg2).map_err(to_err)?;
                            } else if let (Some(reg1), Some(reg2)) = (to_reg64(*r1), to_reg64(*r2))
                            {
                                asm.$func(reg1, reg2).map_err(to_err)?;
                            } else {
                                return Err(AsmError::EncodeError {
                                    line: stmt.line,
                                    message: "Register size mismatch".into(),
                                });
                            }
                        }
                        (Operand::Reg(r1), Operand::Imm(i)) => {
                            if let Some(reg1) = to_reg32(*r1) {
                                asm.$func(reg1, *i as i32).map_err(to_err)?;
                            } else if let Some(reg1) = to_reg64(*r1) {
                                asm.$func(reg1, *i as $imm64_cast).map_err(to_err)?;
                            } else {
                                return Err(AsmError::EncodeError {
                                    line: stmt.line,
                                    message: "Unsupported register size for immediate".into(),
                                });
                            }
                        }
                        (Operand::Reg(r1), Operand::Memory { base, disp }) => {
                            if let (Some(reg1), Some(mem)) = (to_reg64(*r1), to_mem64(*base, *disp))
                            {
                                asm.$func(reg1, mem).map_err(to_err)?;
                            } else if let (Some(reg1), Some(mem)) =
                                (to_reg32(*r1), to_mem64(*base, *disp))
                            {
                                asm.$func(reg1, mem).map_err(to_err)?;
                            } else {
                                return Err(AsmError::EncodeError {
                                    line: stmt.line,
                                    message: "Unsupported mem bounds".into(),
                                });
                            }
                        }
                        (Operand::Memory { base, disp }, Operand::Reg(r2)) => {
                            if let (Some(mem), Some(reg2)) = (to_mem64(*base, *disp), to_reg64(*r2))
                            {
                                asm.$func(mem, reg2).map_err(to_err)?;
                            } else if let (Some(mem), Some(reg2)) =
                                (to_mem64(*base, *disp), to_reg32(*r2))
                            {
                                asm.$func(mem, reg2).map_err(to_err)?;
                            } else {
                                return Err(AsmError::EncodeError {
                                    line: stmt.line,
                                    message: "Unsupported mem bounds".into(),
                                });
                            }
                        }
                        _ => {
                            return Err(AsmError::EncodeError {
                                line: stmt.line,
                                message: "Unsupported operand pairing".into(),
                            })
                        }
                    }
                };
            }

            match op {
                Mnemonic::Mov => bin_op!(mov, i64),
                Mnemonic::Add => bin_op!(add, i32),
                Mnemonic::Sub => bin_op!(sub, i32),
                Mnemonic::Cmp => bin_op!(cmp, i32),
                _ => unreachable!(),
            }

            Ok(())
        }
        _ => Err(AsmError::UnknownMnemonic {
            line: stmt.line,
            mnemonic: format!("{:?}", stmt.mnemonic),
        }),
    }
}
