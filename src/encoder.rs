use crate::error::AsmError;
use crate::resolver::SymbolResolver;
use crate::types::*;
use iced_x86::code_asm::*;
use std::collections::HashMap;

fn to_reg8(r: Reg) -> Option<AsmRegister8> {
    match r {
        Reg::Al => Some(al),
        Reg::Cl => Some(cl),
        Reg::Dl => Some(dl),
        Reg::Bl => Some(bl),
        _ => None,
    }
}

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
        Reg::R8d => Some(r8d),
        Reg::R9d => Some(r9d),
        Reg::R10d => Some(r10d),
        Reg::R11d => Some(r11d),
        Reg::R12d => Some(r12d),
        Reg::R13d => Some(r13d),
        Reg::R14d => Some(r14d),
        Reg::R15d => Some(r15d),
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
        Reg::R8 => Some(r8),
        Reg::R9 => Some(r9),
        Reg::R10 => Some(r10),
        Reg::R11 => Some(r11),
        Reg::R12 => Some(r12),
        Reg::R13 => Some(r13),
        Reg::R14 => Some(r14),
        Reg::R15 => Some(r15),
        _ => None,
    }
}

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
        Reg::R8 | Reg::R8d => Some(r8d),
        Reg::R9 | Reg::R9d => Some(r9d),
        Reg::R10 | Reg::R10d => Some(r10d),
        Reg::R11 | Reg::R11d => Some(r11d),
        Reg::R12 | Reg::R12d => Some(r12d),
        Reg::R13 | Reg::R13d => Some(r13d),
        Reg::R14 | Reg::R14d => Some(r14d),
        Reg::R15 | Reg::R15d => Some(r15d),
        _ => None,
    }
}

fn build_mem(
    size: MemorySize,
    base: Option<Reg>,
    index: Option<Reg>,
    scale: u32,
    disp: i32,
) -> AsmMemoryOperand {
    let mut mem = None;
    if let Some(b) = base {
        if let Some(r) = to_reg64(b) {
            mem = Some(ptr(r));
        } else if let Some(r) = to_reg32(b) {
            mem = Some(ptr(r));
        }
    }
    if let Some(i) = index {
        let s = scale as i32;
        if let Some(r) = to_reg64(i) {
            mem = Some(if let Some(m) = mem {
                m + r * s
            } else {
                ptr(r * s)
            });
        } else if let Some(r) = to_reg32(i) {
            mem = Some(if let Some(m) = mem {
                m + r * s
            } else {
                ptr(r * s)
            });
        }
    }

    let mut final_mem = mem.unwrap_or_else(|| ptr(disp as i64));
    if disp != 0 && (base.is_some() || index.is_some()) {
        final_mem = final_mem + disp;
    }

    match size {
        MemorySize::Byte => byte_ptr(final_mem),
        MemorySize::Word => word_ptr(final_mem),
        MemorySize::Dword => dword_ptr(final_mem),
        MemorySize::Qword => qword_ptr(final_mem),
        MemorySize::Unspecified => final_mem,
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
        Mnemonic::LabelOnly
        | Mnemonic::Global
        | Mnemonic::Text
        | Mnemonic::Data
        | Mnemonic::Align => Ok(()),

        Mnemonic::Ascii | Mnemonic::Asciz => {
            if let Some(Operand::StringBytes(bytes)) = stmt.operands.get(0) {
                asm.db(bytes).map_err(to_err)?;
                if stmt.mnemonic == Mnemonic::Asciz {
                    asm.db(&[0u8]).map_err(to_err)?; // Null terminator
                }
            }
            Ok(())
        }

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

        Mnemonic::Hlt => asm.hlt().map_err(to_err),
        Mnemonic::Int => {
            if let Some(Operand::Imm(i)) = stmt.operands.get(0) {
                asm.int(*i as i32).map_err(to_err)?;
            }
            Ok(())
        }

        Mnemonic::Jmp | Mnemonic::Jcc(_) | Mnemonic::Call => {
            if let Some(Operand::Label(lbl)) = stmt.operands.get(0) {
                if let Some(addr) = resolver.resolve(lbl) {
                    match stmt.mnemonic {
                        Mnemonic::Jmp => asm.jmp(addr).map_err(to_err)?,
                        Mnemonic::Call => asm.call(addr).map_err(to_err)?,
                        Mnemonic::Jcc(cond) => match cond {
                            Condition::O => asm.jo(addr).map_err(to_err)?,
                            Condition::No => asm.jno(addr).map_err(to_err)?,
                            Condition::B => asm.jb(addr).map_err(to_err)?,
                            Condition::Ae => asm.jae(addr).map_err(to_err)?,
                            Condition::E => asm.je(addr).map_err(to_err)?,
                            Condition::Ne => asm.jne(addr).map_err(to_err)?,
                            Condition::Be => asm.jbe(addr).map_err(to_err)?,
                            Condition::A => asm.ja(addr).map_err(to_err)?,
                            Condition::S => asm.js(addr).map_err(to_err)?,
                            Condition::Ns => asm.jns(addr).map_err(to_err)?,
                            Condition::P => asm.jp(addr).map_err(to_err)?,
                            Condition::Np => asm.jnp(addr).map_err(to_err)?,
                            Condition::L => asm.jl(addr).map_err(to_err)?,
                            Condition::Ge => asm.jge(addr).map_err(to_err)?,
                            Condition::Le => asm.jle(addr).map_err(to_err)?,
                            Condition::G => asm.jg(addr).map_err(to_err)?,
                        },
                        _ => unreachable!(),
                    }
                } else {
                    let code_label = *labels
                        .entry(lbl.clone())
                        .or_insert_with(|| asm.create_label());
                    match stmt.mnemonic {
                        Mnemonic::Jmp => asm.jmp(code_label).map_err(to_err)?,
                        Mnemonic::Call => asm.call(code_label).map_err(to_err)?,
                        Mnemonic::Jcc(cond) => match cond {
                            Condition::O => asm.jo(code_label).map_err(to_err)?,
                            Condition::No => asm.jno(code_label).map_err(to_err)?,
                            Condition::B => asm.jb(code_label).map_err(to_err)?,
                            Condition::Ae => asm.jae(code_label).map_err(to_err)?,
                            Condition::E => asm.je(code_label).map_err(to_err)?,
                            Condition::Ne => asm.jne(code_label).map_err(to_err)?,
                            Condition::Be => asm.jbe(code_label).map_err(to_err)?,
                            Condition::A => asm.ja(code_label).map_err(to_err)?,
                            Condition::S => asm.js(code_label).map_err(to_err)?,
                            Condition::Ns => asm.jns(code_label).map_err(to_err)?,
                            Condition::P => asm.jp(code_label).map_err(to_err)?,
                            Condition::Np => asm.jnp(code_label).map_err(to_err)?,
                            Condition::L => asm.jl(code_label).map_err(to_err)?,
                            Condition::Ge => asm.jge(code_label).map_err(to_err)?,
                            Condition::Le => asm.jle(code_label).map_err(to_err)?,
                            Condition::G => asm.jg(code_label).map_err(to_err)?,
                        },
                        _ => unreachable!(),
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

        Mnemonic::Mul | Mnemonic::Div => {
            match &stmt.operands[0] {
                Operand::Reg(r) => {
                    if let Some(reg) = to_reg32(*r) {
                        if stmt.mnemonic == Mnemonic::Mul {
                            asm.mul(reg)
                        } else {
                            asm.div(reg)
                        }
                        .map_err(to_err)?;
                    } else if let Some(reg) = to_reg64(*r) {
                        if stmt.mnemonic == Mnemonic::Mul {
                            asm.mul(reg)
                        } else {
                            asm.div(reg)
                        }
                        .map_err(to_err)?;
                    } else {
                        return Err(AsmError::EncodeError {
                            line: stmt.line,
                            message: "Register size mismatch".into(),
                        });
                    }
                }
                Operand::Memory {
                    size,
                    base,
                    index,
                    scale,
                    disp,
                } => {
                    let mem = build_mem(*size, *base, *index, *scale, *disp);
                    if stmt.mnemonic == Mnemonic::Mul {
                        asm.mul(mem)
                    } else {
                        asm.div(mem)
                    }
                    .map_err(to_err)?;
                }
                _ => {
                    return Err(AsmError::EncodeError {
                        line: stmt.line,
                        message: "Invalid operand for Mul/Div".into(),
                    });
                }
            }
            Ok(())
        }

        Mnemonic::Cmov(cond) => {
            let (o1, o2) = (&stmt.operands[0], &stmt.operands[1]);
            macro_rules! cmov_op {
                ($func:ident) => {
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
                                    message: "Size mismatch".into(),
                                });
                            }
                        }
                        (
                            Operand::Reg(r1),
                            Operand::Memory {
                                size,
                                base,
                                index,
                                scale,
                                disp,
                            },
                        ) => {
                            let mem = build_mem(*size, *base, *index, *scale, *disp);
                            if let Some(reg) = to_reg64(*r1) {
                                asm.$func(reg, mem).map_err(to_err)?;
                            } else if let Some(reg) = to_reg32(*r1) {
                                asm.$func(reg, mem).map_err(to_err)?;
                            }
                        }
                        _ => {
                            return Err(AsmError::EncodeError {
                                line: stmt.line,
                                message: "Invalid CMOV pairing".into(),
                            })
                        }
                    }
                };
            }
            match cond {
                Condition::O => cmov_op!(cmovo),
                Condition::No => cmov_op!(cmovno),
                Condition::B => cmov_op!(cmovb),
                Condition::Ae => cmov_op!(cmovae),
                Condition::E => cmov_op!(cmove),
                Condition::Ne => cmov_op!(cmovne),
                Condition::Be => cmov_op!(cmovbe),
                Condition::A => cmov_op!(cmova),
                Condition::S => cmov_op!(cmovs),
                Condition::Ns => cmov_op!(cmovns),
                Condition::P => cmov_op!(cmovp),
                Condition::Np => cmov_op!(cmovnp),
                Condition::L => cmov_op!(cmovl),
                Condition::Ge => cmov_op!(cmovge),
                Condition::Le => cmov_op!(cmovle),
                Condition::G => cmov_op!(cmovg),
            }
            Ok(())
        }

        Mnemonic::Set(cond) => {
            let o1 = &stmt.operands[0];
            macro_rules! set_op {
                ($func:ident) => {
                    match o1 {
                        Operand::Reg(r) => {
                            if let Some(reg) = to_reg8(*r) {
                                asm.$func(reg).map_err(to_err)?;
                            } else {
                                return Err(AsmError::EncodeError {
                                    line: stmt.line,
                                    message: "Set requires 8-bit reg".into(),
                                });
                            }
                        }
                        Operand::Memory {
                            size,
                            base,
                            index,
                            scale,
                            disp,
                        } => {
                            let mem = build_mem(*size, *base, *index, *scale, *disp);
                            asm.$func(mem).map_err(to_err)?;
                        }
                        _ => {
                            return Err(AsmError::EncodeError {
                                line: stmt.line,
                                message: "Invalid SET target".into(),
                            })
                        }
                    }
                };
            }
            match cond {
                Condition::O => set_op!(seto),
                Condition::No => set_op!(setno),
                Condition::B => set_op!(setb),
                Condition::Ae => set_op!(setae),
                Condition::E => set_op!(sete),
                Condition::Ne => set_op!(setne),
                Condition::Be => set_op!(setbe),
                Condition::A => set_op!(seta),
                Condition::S => set_op!(sets),
                Condition::Ns => set_op!(setns),
                Condition::P => set_op!(setp),
                Condition::Np => set_op!(setnp),
                Condition::L => set_op!(setl),
                Condition::Ge => set_op!(setge),
                Condition::Le => set_op!(setle),
                Condition::G => set_op!(setg),
            }
            Ok(())
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
                    size,
                    base,
                    index,
                    scale,
                    disp,
                },
            ) = (&stmt.operands[0], &stmt.operands[1])
            {
                let mem = build_mem(*size, *base, *index, *scale, *disp);
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
                        size,
                        base,
                        index,
                        scale,
                        disp,
                    },
                ) => {
                    let mem = build_mem(*size, *base, *index, *scale, *disp);
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
                        size,
                        base,
                        index,
                        scale,
                        disp,
                    },
                    Operand::Reg(r2),
                ) => {
                    let mem = build_mem(*size, *base, *index, *scale, *disp);
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
                (
                    Operand::Memory {
                        size,
                        base,
                        index,
                        scale,
                        disp,
                    },
                    Operand::Imm(i),
                ) => {
                    let mem = build_mem(
                        if *size == MemorySize::Unspecified {
                            MemorySize::Dword
                        } else {
                            *size
                        },
                        *base,
                        *index,
                        *scale,
                        *disp,
                    );
                    asm.mov(mem, *i as i32).map_err(to_err)?;
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

        Mnemonic::Test => {
            let (o1, o2) = (&stmt.operands[0], &stmt.operands[1]);
            match (o1, o2) {
                (Operand::Reg(r1), Operand::Reg(r2)) => {
                    if let (Some(reg1), Some(reg2)) = (to_reg32(*r1), to_reg32(*r2)) {
                        asm.test(reg1, reg2).map_err(to_err)?;
                    } else if let (Some(reg1), Some(reg2)) = (to_reg64(*r1), to_reg64(*r2)) {
                        asm.test(reg1, reg2).map_err(to_err)?;
                    } else {
                        return Err(AsmError::EncodeError {
                            line: stmt.line,
                            message: "Register size mismatch".into(),
                        });
                    }
                }
                (Operand::Reg(r1), Operand::Imm(i)) => {
                    if let Some(reg) = to_reg32(*r1) {
                        asm.test(reg, *i as i32).map_err(to_err)?;
                    } else if let Some(reg) = to_reg64(*r1) {
                        asm.test(reg, *i as i32).map_err(to_err)?;
                    }
                }
                (
                    Operand::Reg(r1),
                    Operand::Memory {
                        size,
                        base,
                        index,
                        scale,
                        disp,
                    },
                ) => {
                    let mem = build_mem(*size, *base, *index, *scale, *disp);
                    if let Some(reg) = to_reg64(*r1) {
                        asm.test(mem, reg).map_err(to_err)?;
                    } else if let Some(reg) = to_reg32(*r1) {
                        asm.test(mem, reg).map_err(to_err)?;
                    }
                }
                (
                    Operand::Memory {
                        size,
                        base,
                        index,
                        scale,
                        disp,
                    },
                    Operand::Reg(r2),
                ) => {
                    let mem = build_mem(*size, *base, *index, *scale, *disp);
                    if let Some(reg) = to_reg64(*r2) {
                        asm.test(mem, reg).map_err(to_err)?;
                    } else if let Some(reg) = to_reg32(*r2) {
                        asm.test(mem, reg).map_err(to_err)?;
                    }
                }
                (
                    Operand::Memory {
                        size,
                        base,
                        index,
                        scale,
                        disp,
                    },
                    Operand::Imm(i),
                ) => {
                    let mem = build_mem(
                        if *size == MemorySize::Unspecified {
                            MemorySize::Dword
                        } else {
                            *size
                        },
                        *base,
                        *index,
                        *scale,
                        *disp,
                    );
                    asm.test(mem, *i as i32).map_err(to_err)?;
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

        op @ (Mnemonic::Add
        | Mnemonic::Adc
        | Mnemonic::Sub
        | Mnemonic::Sbb
        | Mnemonic::Cmp
        | Mnemonic::And
        | Mnemonic::Or
        | Mnemonic::Xor) => {
            let (o1, o2) = (&stmt.operands[0], &stmt.operands[1]);
            macro_rules! math_op {
                ($func:ident) => {
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
                            if let Some(reg) = to_reg32(*r1) {
                                asm.$func(reg, *i as i32).map_err(to_err)?;
                            } else if let Some(reg) = to_reg64(*r1) {
                                asm.$func(reg, *i as i32).map_err(to_err)?;
                            }
                        }
                        (
                            Operand::Reg(r1),
                            Operand::Memory {
                                size,
                                base,
                                index,
                                scale,
                                disp,
                            },
                        ) => {
                            let mem = build_mem(*size, *base, *index, *scale, *disp);
                            if let Some(reg) = to_reg64(*r1) {
                                asm.$func(reg, mem).map_err(to_err)?;
                            } else if let Some(reg) = to_reg32(*r1) {
                                asm.$func(reg, mem).map_err(to_err)?;
                            }
                        }
                        (
                            Operand::Memory {
                                size,
                                base,
                                index,
                                scale,
                                disp,
                            },
                            Operand::Reg(r2),
                        ) => {
                            let mem = build_mem(*size, *base, *index, *scale, *disp);
                            if let Some(reg) = to_reg64(*r2) {
                                asm.$func(mem, reg).map_err(to_err)?;
                            } else if let Some(reg) = to_reg32(*r2) {
                                asm.$func(mem, reg).map_err(to_err)?;
                            }
                        }
                        (
                            Operand::Memory {
                                size,
                                base,
                                index,
                                scale,
                                disp,
                            },
                            Operand::Imm(i),
                        ) => {
                            let mem = build_mem(
                                if *size == MemorySize::Unspecified {
                                    MemorySize::Dword
                                } else {
                                    *size
                                },
                                *base,
                                *index,
                                *scale,
                                *disp,
                            );
                            asm.$func(mem, *i as i32).map_err(to_err)?;
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
                Mnemonic::Add => math_op!(add),
                Mnemonic::Adc => math_op!(adc),
                Mnemonic::Sub => math_op!(sub),
                Mnemonic::Sbb => math_op!(sbb),
                Mnemonic::Cmp => math_op!(cmp),
                Mnemonic::And => math_op!(and),
                Mnemonic::Or => math_op!(or),
                Mnemonic::Xor => math_op!(xor),
                _ => unreachable!(),
            }
            Ok(())
        }
    }
}
