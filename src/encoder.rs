use crate::error::AsmError;
use crate::resolver::SymbolResolver;
use crate::types::*;
use iced_x86::code_asm::*;
use std::collections::HashMap;

pub(crate) fn eval_expr(
    expr: &Expr,
    labels: &HashMap<String, i64>,
    resolver: &mut dyn SymbolResolver,
    start_addr: u64,
) -> Result<i64, String> {
    match expr {
        Expr::Number(n) => Ok(*n),
        Expr::Symbol(s) => {
            if let Some(addr) = resolver.resolve(s) {
                Ok(addr as i64)
            } else if let Some(&offset) = labels.get(s) {
                Ok(start_addr as i64 + offset)
            } else {
                Err(format!("Unknown label: {}", s))
            }
        }
        Expr::Add(l, r) => Ok(eval_expr(l, labels, resolver, start_addr)?
            + eval_expr(r, labels, resolver, start_addr)?),
        Expr::Sub(l, r) => Ok(eval_expr(l, labels, resolver, start_addr)?
            - eval_expr(r, labels, resolver, start_addr)?),
    }
}

fn to_reg8(r: Reg) -> Option<AsmRegister8> {
    match r {
        Reg::Al => Some(al),
        Reg::Cl => Some(cl),
        Reg::Dl => Some(dl),
        Reg::Bl => Some(bl),
        Reg::Ah => Some(ah),
        Reg::Ch => Some(ch),
        Reg::Dh => Some(dh),
        Reg::Bh => Some(bh),
        _ => None,
    }
}

fn to_reg16(r: Reg) -> Option<AsmRegister16> {
    match r {
        Reg::Ax => Some(ax),
        Reg::Cx => Some(cx),
        Reg::Dx => Some(dx),
        Reg::Bx => Some(bx),
        Reg::Sp => Some(sp),
        Reg::Bp => Some(bp),
        Reg::Si => Some(si),
        Reg::Di => Some(di),
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
    bitness: u32,
    size: MemorySize,
    base: Option<Reg>,
    index: Option<Reg>,
    scale: u32,
    disp: i64,
) -> AsmMemoryOperand {
    let mut mem = None;
    if let Some(b) = base {
        if bitness == 64 {
            mem = Some(ptr(to_reg64(b).unwrap()));
        } else {
            mem = Some(ptr(to_reg32(b).unwrap()));
        }
    }
    if let Some(i) = index {
        let s = scale as i32;
        if bitness == 64 {
            let r = to_reg64(i).unwrap();
            mem = Some(if let Some(m) = mem {
                m + r * s
            } else {
                ptr(r * s)
            });
        } else {
            let r = to_reg32(i).unwrap();
            mem = Some(if let Some(m) = mem {
                m + r * s
            } else {
                ptr(r * s)
            });
        }
    }

    // iced-x86 ptr() natively accepts the full i64 displacement
    let mut final_mem = mem.unwrap_or_else(|| ptr(disp));

    // If a register is involved, x86 limits displacement to 32-bit (i32)
    if disp != 0 && (base.is_some() || index.is_some()) {
        final_mem = final_mem + disp as i32;
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
    estimated_labels: &HashMap<String, i64>,
    resolver: &mut dyn SymbolResolver,
    start_addr: u64,
) -> Result<(), AsmError> {
    let to_err = |e: IcedError| AsmError::EncodeError {
        line: stmt.line,
        message: e.to_string(),
    };
    let bitness = asm.bitness();

    macro_rules! get_imm {
        ($op:expr) => {
            match $op {
                Operand::Imm(i) => Some(*i),
                Operand::Expr(e) => {
                    crate::encoder::eval_expr(e, estimated_labels, resolver, start_addr).ok()
                }
                _ => None,
            }
        };
    }

    macro_rules! get_mem {
        ($size:expr, $base:expr, $index:expr, $scale:expr, $disp:expr $(,)?) => {{
            crate::encoder::eval_expr($disp, estimated_labels, resolver, start_addr)
                .map_err(|e| AsmError::EncodeError {
                    line: stmt.line,
                    message: e,
                })
                .map(|d| build_mem(bitness, $size, $base, $index, $scale, d))
        }};
    }

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
                    asm.db(&[0u8]).map_err(to_err)?;
                }
            }
            Ok(())
        }
        Mnemonic::Byte => {
            if let Some(i) = stmt.operands.get(0).and_then(|op| get_imm!(op)) {
                asm.db(&[i as u8]).map_err(to_err)?;
            } else {
                return Err(AsmError::EncodeError {
                    line: stmt.line,
                    message: "Byte requires an immediate".into(),
                });
            }
            Ok(())
        }
        Mnemonic::Short => {
            if let Some(i) = stmt.operands.get(0).and_then(|op| get_imm!(op)) {
                asm.dw(&[i as u16]).map_err(to_err)?;
            } else {
                return Err(AsmError::EncodeError {
                    line: stmt.line,
                    message: "Short requires an immediate".into(),
                });
            }
            Ok(())
        }
        Mnemonic::Word => {
            if let Some(i) = stmt.operands.get(0).and_then(|op| get_imm!(op)) {
                asm.dd(&[i as u32]).map_err(to_err)?;
            } else {
                return Err(AsmError::EncodeError {
                    line: stmt.line,
                    message: "Word requires an immediate".into(),
                });
            }
            Ok(())
        }
        Mnemonic::Space => {
            if let Some(i) = stmt.operands.get(0).and_then(|op| get_imm!(op)) {
                if i > 0 {
                    asm.db(&vec![0u8; i as usize]).map_err(to_err)?;
                }
            }
            Ok(())
        }
        Mnemonic::Nop => asm.nop().map_err(to_err),
        Mnemonic::Ret => asm.ret().map_err(to_err),
        Mnemonic::Hlt => asm.hlt().map_err(to_err),
        Mnemonic::Int => {
            if let Some(i) = stmt.operands.get(0).and_then(|op| get_imm!(op)) {
                asm.int(i as i32).map_err(to_err)?;
            } else {
                return Err(AsmError::EncodeError {
                    line: stmt.line,
                    message: "Int requires an immediate".into(),
                });
            }
            Ok(())
        }
        Mnemonic::Jmp | Mnemonic::Jcc(_) | Mnemonic::Call => {
            let get_lbl = || -> Option<String> {
                match stmt.operands.get(0) {
                    Some(Operand::Label(lbl)) => Some(lbl.clone()),
                    Some(Operand::Expr(Expr::Symbol(lbl))) => Some(lbl.clone()),
                    _ => None,
                }
            };
            macro_rules! do_jump {
                ($tgt:expr) => {
                    match stmt.mnemonic {
                        Mnemonic::Jmp => asm.jmp($tgt).map_err(to_err)?,
                        Mnemonic::Call => asm.call($tgt).map_err(to_err)?,
                        Mnemonic::Jcc(cond) => match cond {
                            Condition::O => asm.jo($tgt).map_err(to_err)?,
                            Condition::No => asm.jno($tgt).map_err(to_err)?,
                            Condition::B => asm.jb($tgt).map_err(to_err)?,
                            Condition::Ae => asm.jae($tgt).map_err(to_err)?,
                            Condition::E => asm.je($tgt).map_err(to_err)?,
                            Condition::Ne => asm.jne($tgt).map_err(to_err)?,
                            Condition::Be => asm.jbe($tgt).map_err(to_err)?,
                            Condition::A => asm.ja($tgt).map_err(to_err)?,
                            Condition::S => asm.js($tgt).map_err(to_err)?,
                            Condition::Ns => asm.jns($tgt).map_err(to_err)?,
                            Condition::P => asm.jp($tgt).map_err(to_err)?,
                            Condition::Np => asm.jnp($tgt).map_err(to_err)?,
                            Condition::L => asm.jl($tgt).map_err(to_err)?,
                            Condition::Ge => asm.jge($tgt).map_err(to_err)?,
                            Condition::Le => asm.jle($tgt).map_err(to_err)?,
                            Condition::G => asm.jg($tgt).map_err(to_err)?,
                        },
                        _ => unreachable!(),
                    };
                };
            }
            if let Some(lbl) = get_lbl() {
                if let Some(addr) = resolver.resolve(&lbl) {
                    do_jump!(addr);
                } else {
                    let code_label = *labels
                        .entry(lbl.clone())
                        .or_insert_with(|| asm.create_label());
                    do_jump!(code_label);
                }
            } else if let Some(i) = get_imm!(&stmt.operands[0]) {
                do_jump!(i as u64);
            } else {
                return Err(AsmError::EncodeError {
                    line: stmt.line,
                    message: "Label or address required".into(),
                });
            }
            Ok(())
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
                    } else if let Some(reg) = to_reg16(*r) {
                        if stmt.mnemonic == Mnemonic::Mul {
                            asm.mul(reg)
                        } else {
                            asm.div(reg)
                        }
                        .map_err(to_err)?;
                    } else if let Some(reg) = to_reg8(*r) {
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
                    let mem = get_mem!(*size, *base, *index, *scale, disp)?;
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
                            } else if let (Some(reg1), Some(reg2)) = (to_reg16(*r1), to_reg16(*r2))
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
                            let mem = get_mem!(*size, *base, *index, *scale, disp)?;
                            if let Some(reg) = to_reg64(*r1) {
                                asm.$func(reg, mem).map_err(to_err)?;
                            } else if let Some(reg) = to_reg32(*r1) {
                                asm.$func(reg, mem).map_err(to_err)?;
                            } else if let Some(reg) = to_reg16(*r1) {
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
            macro_rules! set_op {
                ($func:ident) => {
                    match &stmt.operands[0] {
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
                            asm.$func(get_mem!(*size, *base, *index, *scale, disp)?)
                                .map_err(to_err)?;
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
            } else if let Some(reg) = to_reg16(*r) {
                if stmt.mnemonic == Mnemonic::Inc {
                    asm.inc(reg)
                } else {
                    asm.dec(reg)
                }
                .map_err(to_err)?;
            } else if let Some(reg) = to_reg8(*r) {
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
                    } else if let Some(reg) = to_reg16(*r) {
                        if is_push { asm.push(reg) } else { asm.pop(reg) }.map_err(to_err)?;
                    }
                }
                op => {
                    if let Some(i) = get_imm!(op) {
                        if is_push {
                            asm.push(i as i32).map_err(to_err)?;
                        } else {
                            return Err(AsmError::EncodeError {
                                line: stmt.line,
                                message: "Cannot pop immediate".into(),
                            });
                        }
                    } else {
                        return Err(AsmError::EncodeError {
                            line: stmt.line,
                            message: "Unsupported operand pairing".into(),
                        });
                    }
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
                let mem = get_mem!(*size, *base, *index, *scale, disp)?;
                if let Some(reg) = to_reg64(*r1) {
                    asm.lea(reg, mem).map_err(to_err)?;
                } else if let Some(reg) = to_reg32(*r1) {
                    asm.lea(reg, mem).map_err(to_err)?;
                } else if let Some(reg) = to_reg16(*r1) {
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
            match (&stmt.operands[0], &stmt.operands[1]) {
                (Operand::Reg(r1), Operand::Reg(r2)) => {
                    if let (Some(reg1), Some(reg2)) = (to_reg32(*r1), to_reg32(*r2)) {
                        asm.mov(reg1, reg2).map_err(to_err)?;
                    } else if let (Some(reg1), Some(reg2)) = (to_reg64(*r1), to_reg64(*r2)) {
                        asm.mov(reg1, reg2).map_err(to_err)?;
                    } else if let (Some(reg1), Some(reg2)) = (to_reg16(*r1), to_reg16(*r2)) {
                        asm.mov(reg1, reg2).map_err(to_err)?;
                    } else if let (Some(reg1), Some(reg2)) = (to_reg8(*r1), to_reg8(*r2)) {
                        asm.mov(reg1, reg2).map_err(to_err)?;
                    } else {
                        return Err(AsmError::EncodeError {
                            line: stmt.line,
                            message: "Register size mismatch".into(),
                        });
                    }
                }
                (Operand::Reg(r1), op) => {
                    if let Some(i) = get_imm!(op) {
                        if bitness == 64
                            && to_reg64(*r1).is_some()
                            && i >= 0
                            && i <= u32::MAX as i64
                        {
                            asm.mov(get_sub_reg32(*r1).unwrap(), i as i32)
                                .map_err(to_err)?;
                        } else if let Some(reg) = to_reg64(*r1) {
                            asm.mov(reg, i as i64).map_err(to_err)?;
                        } else if let Some(reg) = to_reg32(*r1) {
                            asm.mov(reg, i as i32).map_err(to_err)?;
                        } else if let Some(reg) = to_reg16(*r1) {
                            asm.mov(reg, i as i32).map_err(to_err)?;
                        } else if let Some(reg) = to_reg8(*r1) {
                            asm.mov(reg, i as i32).map_err(to_err)?;
                        }
                    } else if let Operand::Memory {
                        size,
                        base,
                        index,
                        scale,
                        disp,
                    } = op
                    {
                        let mem = get_mem!(*size, *base, *index, *scale, disp)?;
                        if let Some(reg) = to_reg64(*r1) {
                            asm.mov(reg, mem).map_err(to_err)?;
                        } else if let Some(reg) = to_reg32(*r1) {
                            asm.mov(reg, mem).map_err(to_err)?;
                        } else if let Some(reg) = to_reg16(*r1) {
                            asm.mov(reg, mem).map_err(to_err)?;
                        } else if let Some(reg) = to_reg8(*r1) {
                            asm.mov(reg, mem).map_err(to_err)?;
                        } else {
                            return Err(AsmError::EncodeError {
                                line: stmt.line,
                                message: "Register size mismatch".into(),
                            });
                        }
                    } else {
                        return Err(AsmError::EncodeError {
                            line: stmt.line,
                            message: "Unsupported operand pairing".into(),
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
                    let mem = get_mem!(*size, *base, *index, *scale, disp)?;
                    if let Some(reg) = to_reg64(*r2) {
                        asm.mov(mem, reg).map_err(to_err)?;
                    } else if let Some(reg) = to_reg32(*r2) {
                        asm.mov(mem, reg).map_err(to_err)?;
                    } else if let Some(reg) = to_reg16(*r2) {
                        asm.mov(mem, reg).map_err(to_err)?;
                    } else if let Some(reg) = to_reg8(*r2) {
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
                    op,
                ) => {
                    if let Some(i) = get_imm!(op) {
                        let mem = get_mem!(
                            if *size == MemorySize::Unspecified {
                                MemorySize::Dword
                            } else {
                                *size
                            },
                            *base,
                            *index,
                            *scale,
                            disp,
                        )?;
                        asm.mov(mem, i as i32).map_err(to_err)?;
                    } else {
                        return Err(AsmError::EncodeError {
                            line: stmt.line,
                            message: "Unsupported operand pairing".into(),
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
        Mnemonic::Test => {
            match (&stmt.operands[0], &stmt.operands[1]) {
                (Operand::Reg(r1), Operand::Reg(r2)) => {
                    if let (Some(reg1), Some(reg2)) = (to_reg32(*r1), to_reg32(*r2)) {
                        asm.test(reg1, reg2).map_err(to_err)?;
                    } else if let (Some(reg1), Some(reg2)) = (to_reg64(*r1), to_reg64(*r2)) {
                        asm.test(reg1, reg2).map_err(to_err)?;
                    } else if let (Some(reg1), Some(reg2)) = (to_reg16(*r1), to_reg16(*r2)) {
                        asm.test(reg1, reg2).map_err(to_err)?;
                    } else if let (Some(reg1), Some(reg2)) = (to_reg8(*r1), to_reg8(*r2)) {
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
                    } else if let Some(reg) = to_reg16(*r1) {
                        asm.test(reg, *i as i32).map_err(to_err)?;
                    } else if let Some(reg) = to_reg8(*r1) {
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
                    let mem = get_mem!(*size, *base, *index, *scale, disp)?;
                    if let Some(reg) = to_reg64(*r1) {
                        asm.test(mem, reg).map_err(to_err)?;
                    } else if let Some(reg) = to_reg32(*r1) {
                        asm.test(mem, reg).map_err(to_err)?;
                    } else if let Some(reg) = to_reg16(*r1) {
                        asm.test(mem, reg).map_err(to_err)?;
                    } else if let Some(reg) = to_reg8(*r1) {
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
                    let mem = get_mem!(*size, *base, *index, *scale, disp)?;
                    if let Some(reg) = to_reg64(*r2) {
                        asm.test(mem, reg).map_err(to_err)?;
                    } else if let Some(reg) = to_reg32(*r2) {
                        asm.test(mem, reg).map_err(to_err)?;
                    } else if let Some(reg) = to_reg16(*r2) {
                        asm.test(mem, reg).map_err(to_err)?;
                    } else if let Some(reg) = to_reg8(*r2) {
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
                    let mem = get_mem!(
                        if *size == MemorySize::Unspecified {
                            MemorySize::Dword
                        } else {
                            *size
                        },
                        *base,
                        *index,
                        *scale,
                        disp,
                    )?;
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
            macro_rules! math_op {
                ($func:ident) => {
                    match (&stmt.operands[0], &stmt.operands[1]) {
                        (Operand::Reg(r1), Operand::Reg(r2)) => {
                            if let (Some(reg1), Some(reg2)) = (to_reg32(*r1), to_reg32(*r2)) {
                                asm.$func(reg1, reg2).map_err(to_err)?;
                            } else if let (Some(reg1), Some(reg2)) = (to_reg64(*r1), to_reg64(*r2))
                            {
                                asm.$func(reg1, reg2).map_err(to_err)?;
                            } else if let (Some(reg1), Some(reg2)) = (to_reg16(*r1), to_reg16(*r2))
                            {
                                asm.$func(reg1, reg2).map_err(to_err)?;
                            } else if let (Some(reg1), Some(reg2)) = (to_reg8(*r1), to_reg8(*r2)) {
                                asm.$func(reg1, reg2).map_err(to_err)?;
                            } else {
                                return Err(AsmError::EncodeError {
                                    line: stmt.line,
                                    message: "Register size mismatch".into(),
                                });
                            }
                        }
                        (Operand::Reg(r1), op) => {
                            if let Some(i) = get_imm!(op) {
                                if let Some(reg) = to_reg32(*r1) {
                                    asm.$func(reg, i as i32).map_err(to_err)?;
                                } else if let Some(reg) = to_reg64(*r1) {
                                    asm.$func(reg, i as i32).map_err(to_err)?;
                                } else if let Some(reg) = to_reg16(*r1) {
                                    asm.$func(reg, i as i32).map_err(to_err)?;
                                } else if let Some(reg) = to_reg8(*r1) {
                                    asm.$func(reg, i as i32).map_err(to_err)?;
                                }
                            } else if let Operand::Memory {
                                size,
                                base,
                                index,
                                scale,
                                disp,
                            } = op
                            {
                                let mem = get_mem!(*size, *base, *index, *scale, disp)?;
                                if let Some(reg) = to_reg64(*r1) {
                                    asm.$func(reg, mem).map_err(to_err)?;
                                } else if let Some(reg) = to_reg32(*r1) {
                                    asm.$func(reg, mem).map_err(to_err)?;
                                } else if let Some(reg) = to_reg16(*r1) {
                                    asm.$func(reg, mem).map_err(to_err)?;
                                } else if let Some(reg) = to_reg8(*r1) {
                                    asm.$func(reg, mem).map_err(to_err)?;
                                }
                            } else {
                                return Err(AsmError::EncodeError {
                                    line: stmt.line,
                                    message: "Unsupported operand pairing".into(),
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
                            let mem = get_mem!(*size, *base, *index, *scale, disp)?;
                            if let Some(reg) = to_reg64(*r2) {
                                asm.$func(mem, reg).map_err(to_err)?;
                            } else if let Some(reg) = to_reg32(*r2) {
                                asm.$func(mem, reg).map_err(to_err)?;
                            } else if let Some(reg) = to_reg16(*r2) {
                                asm.$func(mem, reg).map_err(to_err)?;
                            } else if let Some(reg) = to_reg8(*r2) {
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
                            op,
                        ) => {
                            if let Some(i) = get_imm!(op) {
                                let mem = get_mem!(
                                    if *size == MemorySize::Unspecified {
                                        MemorySize::Dword
                                    } else {
                                        *size
                                    },
                                    *base,
                                    *index,
                                    *scale,
                                    disp,
                                )?;
                                asm.$func(mem, i as i32).map_err(to_err)?;
                            } else {
                                return Err(AsmError::EncodeError {
                                    line: stmt.line,
                                    message: "Unsupported operand pairing".into(),
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
