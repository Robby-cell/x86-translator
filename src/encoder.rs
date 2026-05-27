use crate::error::AsmError;
use crate::resolver::SymbolResolver;
use crate::types::*;
use iced_x86::code_asm::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(crate) struct EvalResult {
    pub absolute: i64,
    pub unresolved_label: Option<(String, i64)>, // (label_name, offset)
}

pub(crate) fn eval_expr(
    expr: &Expr,
    labels: &HashMap<String, i64>,
    resolver: &mut dyn SymbolResolver,
    start_addr: u64,
) -> Result<EvalResult, String> {
    match expr {
        Expr::Number(n) => Ok(EvalResult {
            absolute: *n,
            unresolved_label: None,
        }),
        Expr::Symbol(s) => {
            if let Some(addr) = resolver.resolve(s) {
                Ok(EvalResult {
                    absolute: addr as i64,
                    unresolved_label: None,
                })
            } else if let Some(&offset) = labels.get(s) {
                Ok(EvalResult {
                    absolute: start_addr as i64 + offset,
                    unresolved_label: Some((s.clone(), 0)),
                })
            } else {
                Err(format!("Unknown label: {}", s))
            }
        }
        Expr::Add(l, r) => {
            let left = eval_expr(l, labels, resolver, start_addr)?;
            let right = eval_expr(r, labels, resolver, start_addr)?;

            let unresolved = match (left.unresolved_label, right.unresolved_label) {
                (Some((s, o)), None) => Some((s, o + right.absolute)),
                (None, Some((s, o))) => Some((s, o + left.absolute)),
                (Some(_), Some(_)) => None, // Adding two labels defaults back to absolute
                (None, None) => None,
            };

            Ok(EvalResult {
                absolute: left.absolute + right.absolute,
                unresolved_label: unresolved,
            })
        }
        Expr::Sub(l, r) => {
            let left = eval_expr(l, labels, resolver, start_addr)?;
            let right = eval_expr(r, labels, resolver, start_addr)?;

            let unresolved = match (left.unresolved_label, right.unresolved_label) {
                (Some((s, o)), None) => Some((s, o - right.absolute)),
                (None, Some(_)) => None,
                (Some((s1, _)), Some((s2, _))) if s1 == s2 => None, // Same label subtracted defaults back to absolute
                (Some(_), Some(_)) => None,
                (None, None) => None,
            };

            Ok(EvalResult {
                absolute: left.absolute - right.absolute,
                unresolved_label: unresolved,
            })
        }
    }
}

fn size_from_reg(r: Reg) -> MemorySize {
    match r {
        Reg::Al | Reg::Cl | Reg::Dl | Reg::Bl => MemorySize::Byte,
        Reg::Ax | Reg::Cx | Reg::Dx | Reg::Bx => MemorySize::Word,
        Reg::Eax
        | Reg::Ecx
        | Reg::Edx
        | Reg::Ebx
        | Reg::Esp
        | Reg::Ebp
        | Reg::Esi
        | Reg::Edi
        | Reg::R8d
        | Reg::R9d
        | Reg::R10d
        | Reg::R11d
        | Reg::R12d
        | Reg::R13d
        | Reg::R14d
        | Reg::R15d => MemorySize::Dword,
        _ => MemorySize::Qword,
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
    asm: &mut CodeAssembler,
    code_labels: &mut HashMap<String, CodeLabel>,
    bitness: u32,
    size: MemorySize,
    base: Option<Reg>,
    index: Option<Reg>,
    scale: u32,
    disp: EvalResult,
    pic: bool,
) -> AsmMemoryOperand {
    let mut mem = None;
    if let Some(b) = base {
        if bitness == 64 {
            mem = to_reg64(b).map(|r| ptr(r));
        } else {
            mem = to_reg32(b).map(|r| ptr(r));
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

    let final_mem = if let Some(m) = mem {
        // If a register is involved, x86 limits displacement to 32-bit (i32)
        let d = disp.absolute;
        if d != 0 { m + d as i32 } else { m }
    } else {
        // No base/index. Check if PIC is enabled and we have an unresolved label.
        if pic && bitness == 64 {
            if let Some((lbl, offset)) = disp.unresolved_label {
                let code_label = *code_labels.entry(lbl).or_insert_with(|| asm.create_label());
                let mut m = ptr(code_label);
                if offset != 0 {
                    m = m + offset as i32;
                }
                m
            } else {
                ptr(disp.absolute)
            }
        } else {
            ptr(disp.absolute)
        }
    };

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
    pic: bool,
) -> Result<(), AsmError> {
    let to_err = |e: IcedError| AsmError::EncodeError {
        line: stmt.line,
        message: e.to_string(),
    };
    let bitness = asm.bitness();

    // Clones the operands so we can mutate them to mimic MASM behavior
    let mut operands = stmt.operands.clone();

    // Only automatically dereference labels on instructions that manipulate data
    let is_data_op = matches!(
        stmt.mnemonic,
        Mnemonic::Mov
            | Mnemonic::Add
            | Mnemonic::Adc
            | Mnemonic::Sub
            | Mnemonic::Sbb
            | Mnemonic::Cmp
            | Mnemonic::Test
            | Mnemonic::And
            | Mnemonic::Or
            | Mnemonic::Xor
            | Mnemonic::Mul
            | Mnemonic::Div
            | Mnemonic::Lea
            | Mnemonic::Inc
            | Mnemonic::Dec
            | Mnemonic::Push
            | Mnemonic::Pop
            | Mnemonic::Set(_)
            | Mnemonic::Cmov(_)
    );

    for op in &mut operands {
        if is_data_op {
            if let Operand::Expr(e) = op {
                if e.has_symbol() {
                    // Turn bare labels automatically into Memory Operands!
                    *op = Operand::Memory {
                        size: MemorySize::Unspecified,
                        base: None,
                        index: None,
                        scale: 1,
                        disp: e.clone(),
                    };
                }
            }
        }

        if let Operand::Offset(e) = op {
            // Strip the `offset` wrapper and treat it as a raw number/address
            *op = Operand::Expr(e.clone());
        }
    }

    macro_rules! get_imm {
        ($op:expr) => {
            match $op {
                Operand::Imm(i) => Some(*i),
                Operand::Expr(e) => {
                    crate::encoder::eval_expr(e, estimated_labels, resolver, start_addr)
                        .ok()
                        .map(|res| res.absolute)
                }
                _ => None,
            }
        };
    }

    macro_rules! get_mem {
        ($size:expr, $base:expr, $index:expr, $scale:expr, $disp:expr, $infer_from_reg:expr $(,)?) => {{
            crate::encoder::eval_expr($disp, estimated_labels, resolver, start_addr)
                .map_err(|e| AsmError::EncodeError {
                    line: stmt.line,
                    message: e,
                })
                .map(|d| {
                    let mut sz = $size;
                    if sz == MemorySize::Unspecified {
                        if let Some(r) = $infer_from_reg {
                            sz = size_from_reg(r);
                        }
                    }
                    build_mem(asm, labels, bitness, sz, $base, $index, $scale, d, pic)
                })
        }};
    }

    match stmt.mnemonic {
        Mnemonic::LabelOnly
        | Mnemonic::Global
        | Mnemonic::Text
        | Mnemonic::Data
        | Mnemonic::Align => Ok(()),
        Mnemonic::Ascii | Mnemonic::Asciz => {
            if let Some(Operand::StringBytes(bytes)) = operands.get(0) {
                asm.db(bytes).map_err(to_err)?;
                if stmt.mnemonic == Mnemonic::Asciz {
                    asm.db(&[0u8]).map_err(to_err)?;
                }
            }
            Ok(())
        }
        Mnemonic::Byte => {
            if let Some(i) = operands.get(0).and_then(|op| get_imm!(op)) {
                asm.db(&[i as u8]).map_err(to_err)?;
            }
            Ok(())
        }
        Mnemonic::Short => {
            if let Some(i) = operands.get(0).and_then(|op| get_imm!(op)) {
                asm.dw(&[i as u16]).map_err(to_err)?;
            }
            Ok(())
        }
        Mnemonic::Word => {
            if let Some(i) = operands.get(0).and_then(|op| get_imm!(op)) {
                asm.dd(&[i as u32]).map_err(to_err)?;
            }
            Ok(())
        }
        Mnemonic::Space => {
            if let Some(i) = operands.get(0).and_then(|op| get_imm!(op)) {
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
            if let Some(i) = operands.get(0).and_then(|op| get_imm!(op)) {
                asm.int(i as i32).map_err(to_err)?;
            }
            Ok(())
        }

        Mnemonic::Jmp | Mnemonic::Jcc(_) | Mnemonic::Call => {
            let get_lbl = || -> Option<String> {
                match operands.get(0) {
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
                    }
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
            } else if let Some(i) = get_imm!(&operands[0]) {
                do_jump!(i as u64);
            }
            Ok(())
        }

        Mnemonic::Mul | Mnemonic::Div => {
            match &operands[0] {
                Operand::Reg(r) => {
                    if let Some(reg) = to_reg64(*r) {
                        if stmt.mnemonic == Mnemonic::Mul {
                            asm.mul(reg)
                        } else {
                            asm.div(reg)
                        }
                        .map_err(to_err)?;
                    } else if let Some(reg) = to_reg32(*r) {
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
                    }
                }
                Operand::Memory {
                    size,
                    base,
                    index,
                    scale,
                    disp,
                } => {
                    let mem = get_mem!(*size, *base, *index, *scale, disp, None)?;
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
                        message: "Unsupported pairing".into(),
                    });
                }
            }
            Ok(())
        }

        Mnemonic::Cmov(cond) => {
            let (o1, o2) = (&operands[0], &operands[1]);
            macro_rules! cmov_op {
                ($func:ident) => {
                    match (o1, o2) {
                        (Operand::Reg(r1), Operand::Reg(r2)) => {
                            if let (Some(reg1), Some(reg2)) = (to_reg64(*r1), to_reg64(*r2)) {
                                asm.$func(reg1, reg2).map_err(to_err)?;
                            } else if let (Some(reg1), Some(reg2)) = (to_reg32(*r1), to_reg32(*r2))
                            {
                                asm.$func(reg1, reg2).map_err(to_err)?;
                            } else if let (Some(reg1), Some(reg2)) = (to_reg16(*r1), to_reg16(*r2))
                            {
                                asm.$func(reg1, reg2).map_err(to_err)?;
                            } else {
                                return Err(AsmError::EncodeError {
                                    line: stmt.line,
                                    message: "Register size mismatch".into(),
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
                            let mem = get_mem!(*size, *base, *index, *scale, disp, Some(*r1))?;
                            if let Some(reg) = to_reg64(*r1) {
                                asm.$func(reg, mem).map_err(to_err)?;
                            } else if let Some(reg) = to_reg32(*r1) {
                                asm.$func(reg, mem).map_err(to_err)?;
                            } else if let Some(reg) = to_reg16(*r1) {
                                asm.$func(reg, mem).map_err(to_err)?;
                            } else {
                                return Err(AsmError::EncodeError {
                                    line: stmt.line,
                                    message: "Invalid register".into(),
                                });
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
                    match &operands[0] {
                        Operand::Reg(r) => {
                            if let Some(reg) = to_reg8(*r) {
                                asm.$func(reg).map_err(to_err)?;
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
                            let mem = get_mem!(*size, *base, *index, *scale, disp, None)?;
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
            let r = match &operands[0] {
                Operand::Reg(r) => r,
                _ => {
                    return Err(AsmError::EncodeError {
                        line: stmt.line,
                        message: "Register required".into(),
                    });
                }
            };
            if let Some(reg) = to_reg64(*r) {
                if stmt.mnemonic == Mnemonic::Inc {
                    asm.inc(reg)
                } else {
                    asm.dec(reg)
                }
                .map_err(to_err)?;
            } else if let Some(reg) = to_reg32(*r) {
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
            match &operands[0] {
                Operand::Reg(r) => {
                    if let Some(reg) = to_reg64(*r) {
                        if is_push { asm.push(reg) } else { asm.pop(reg) }.map_err(to_err)?;
                    } else if let Some(reg) = to_reg32(*r) {
                        if is_push { asm.push(reg) } else { asm.pop(reg) }.map_err(to_err)?;
                    } else {
                        return Err(AsmError::EncodeError {
                            line: stmt.line,
                            message: "Invalid register".into(),
                        });
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
            ) = (&operands[0], &operands[1])
            {
                let mem = get_mem!(*size, *base, *index, *scale, disp, Some(*r1))?;
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
            let (o1, o2) = (&operands[0], &operands[1]);
            match (o1, o2) {
                (Operand::Reg(r1), Operand::Reg(r2)) => {
                    if let (Some(reg1), Some(reg2)) = (to_reg64(*r1), to_reg64(*r2)) {
                        asm.mov(reg1, reg2).map_err(to_err)?;
                    } else if let (Some(reg1), Some(reg2)) = (to_reg32(*r1), to_reg32(*r2)) {
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
                            let sub_reg = get_sub_reg32(*r1).unwrap();
                            asm.mov(sub_reg, i as i32).map_err(to_err)?;
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
                        let mem = get_mem!(*size, *base, *index, *scale, disp, Some(*r1))?;
                        if let Some(reg) = to_reg64(*r1) {
                            asm.mov(reg, mem).map_err(to_err)?;
                        } else if let Some(reg) = to_reg32(*r1) {
                            asm.mov(reg, mem).map_err(to_err)?;
                        } else if let Some(reg) = to_reg16(*r1) {
                            asm.mov(reg, mem).map_err(to_err)?;
                        } else if let Some(reg) = to_reg8(*r1) {
                            asm.mov(reg, mem).map_err(to_err)?;
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
                    let mem = get_mem!(*size, *base, *index, *scale, disp, Some(*r2))?;
                    if let Some(reg) = to_reg64(*r2) {
                        asm.mov(mem, reg).map_err(to_err)?;
                    } else if let Some(reg) = to_reg32(*r2) {
                        asm.mov(mem, reg).map_err(to_err)?;
                    } else if let Some(reg) = to_reg16(*r2) {
                        asm.mov(mem, reg).map_err(to_err)?;
                    } else if let Some(reg) = to_reg8(*r2) {
                        asm.mov(mem, reg).map_err(to_err)?;
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
                        let sz = if *size == MemorySize::Unspecified {
                            MemorySize::Dword
                        } else {
                            *size
                        };
                        let mem = get_mem!(sz, *base, *index, *scale, disp, None)?;
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
                    let mem = get_mem!(*size, *base, *index, *scale, disp, Some(*r1))?;
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
                    let mem = get_mem!(*size, *base, *index, *scale, disp, Some(*r2))?;
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
                        None,
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
            let (o1, o2) = (&operands[0], &operands[1]);
            macro_rules! math_op {
                ($func:ident) => {
                    match (o1, o2) {
                        (Operand::Reg(r1), Operand::Reg(r2)) => {
                            if let (Some(reg1), Some(reg2)) = (to_reg64(*r1), to_reg64(*r2)) {
                                asm.$func(reg1, reg2).map_err(to_err)?;
                            } else if let (Some(reg1), Some(reg2)) = (to_reg32(*r1), to_reg32(*r2))
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
                                if let Some(reg) = to_reg64(*r1) {
                                    asm.$func(reg, i as i32).map_err(to_err)?;
                                } else if let Some(reg) = to_reg32(*r1) {
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
                                let mem = get_mem!(*size, *base, *index, *scale, disp, Some(*r1))?;
                                if matches!(stmt.mnemonic, Mnemonic::Test) {
                                    if let Some(reg) = to_reg64(*r1) {
                                        asm.test(mem, reg).map_err(to_err)?;
                                    } else if let Some(reg) = to_reg32(*r1) {
                                        asm.test(mem, reg).map_err(to_err)?;
                                    } else if let Some(reg) = to_reg16(*r1) {
                                        asm.test(mem, reg).map_err(to_err)?;
                                    } else if let Some(reg) = to_reg8(*r1) {
                                        asm.test(mem, reg).map_err(to_err)?;
                                    }
                                } else {
                                    if let Some(reg) = to_reg64(*r1) {
                                        asm.$func(reg, mem).map_err(to_err)?;
                                    } else if let Some(reg) = to_reg32(*r1) {
                                        asm.$func(reg, mem).map_err(to_err)?;
                                    } else if let Some(reg) = to_reg16(*r1) {
                                        asm.$func(reg, mem).map_err(to_err)?;
                                    } else if let Some(reg) = to_reg8(*r1) {
                                        asm.$func(reg, mem).map_err(to_err)?;
                                    }
                                }
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
                            let mem = get_mem!(*size, *base, *index, *scale, disp, Some(*r2))?;
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
                                let sz = if *size == MemorySize::Unspecified {
                                    MemorySize::Dword
                                } else {
                                    *size
                                };
                                let mem = get_mem!(sz, *base, *index, *scale, disp, None)?;
                                asm.$func(mem, i as i32).map_err(to_err)?;
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
