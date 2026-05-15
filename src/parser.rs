use crate::error::AsmError;
use crate::types::*;
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag_no_case, take_while, take_while1},
    character::complete::{char, digit1, hex_digit1, space0},
    combinator::{map, map_res, opt, recognize},
    sequence::{preceded, terminated},
};

fn sp(input: &str) -> IResult<&str, &str> {
    space0(input)
}

fn register(input: &str) -> IResult<&str, Reg> {
    let (input, name) = take_while1(|c: char| c.is_alphanumeric()).parse(input)?;
    let reg = match name.to_lowercase().as_str() {
        "al" => Reg::Al,
        "cl" => Reg::Cl,
        "dl" => Reg::Dl,
        "bl" => Reg::Bl,
        "ax" => Reg::Ax,
        "cx" => Reg::Cx,
        "dx" => Reg::Dx,
        "bx" => Reg::Bx,
        "eax" => Reg::Eax,
        "ecx" => Reg::Ecx,
        "edx" => Reg::Edx,
        "ebx" => Reg::Ebx,
        "esp" => Reg::Esp,
        "ebp" => Reg::Ebp,
        "esi" => Reg::Esi,
        "edi" => Reg::Edi,
        "r8d" => Reg::R8d,
        "r9d" => Reg::R9d,
        "r10d" => Reg::R10d,
        "r11d" => Reg::R11d,
        "r12d" => Reg::R12d,
        "r13d" => Reg::R13d,
        "r14d" => Reg::R14d,
        "r15d" => Reg::R15d,
        "rax" => Reg::Rax,
        "rcx" => Reg::Rcx,
        "rdx" => Reg::Rdx,
        "rbx" => Reg::Rbx,
        "rsp" => Reg::Rsp,
        "rbp" => Reg::Rbp,
        "rsi" => Reg::Rsi,
        "rdi" => Reg::Rdi,
        "r8" => Reg::R8,
        "r9" => Reg::R9,
        "r10" => Reg::R10,
        "r11" => Reg::R11,
        "r12" => Reg::R12,
        "r13" => Reg::R13,
        "r14" => Reg::R14,
        "r15" => Reg::R15,
        _ => {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }
    };
    Ok((input, reg))
}

fn immediate(input: &str) -> IResult<&str, i64> {
    preceded(
        opt(char('#')),
        map(
            (
                opt(char('-')),
                alt((
                    preceded(
                        tag_no_case("0x"),
                        map_res(hex_digit1, |h: &str| i64::from_str_radix(h, 16)),
                    ),
                    map_res(digit1, |d: &str| d.parse::<i64>()),
                )),
            ),
            |(minus, val)| if minus.is_some() { -val } else { val },
        ),
    )
    .parse(input)
}

fn unsigned_imm(input: &str) -> IResult<&str, i64> {
    alt((
        preceded(
            tag_no_case("0x"),
            map_res(hex_digit1, |h: &str| i64::from_str_radix(h, 16)),
        ),
        map_res(digit1, |d: &str| d.parse::<i64>()),
    ))
    .parse(input)
}

fn label_name(input: &str) -> IResult<&str, String> {
    map(
        recognize((
            take_while1(|c: char| c.is_alphabetic() || c == '_' || c == '.'),
            take_while(|c: char| c.is_alphanumeric() || c == '_' || c == '.'),
        )),
        |s: &str| s.to_string(),
    )
    .parse(input)
}

#[derive(Debug, Clone)]
enum MemTerm {
    Reg(Reg),
    Scale(Reg, u32),
    Disp(i32),
}

fn mem_term(input: &str) -> IResult<&str, MemTerm> {
    alt((
        map(
            (
                register,
                sp,
                char('*'),
                sp,
                alt((char('1'), char('2'), char('4'), char('8'))),
            ),
            |(r, _, _, _, s)| MemTerm::Scale(r, s.to_digit(10).unwrap()),
        ),
        map(register, MemTerm::Reg),
        map(unsigned_imm, |i| MemTerm::Disp(i as i32)),
    ))
    .parse(input)
}

fn memory(input: &str) -> IResult<&str, Operand> {
    let (input, size_str) = opt(alt((
        tag_no_case("qword ptr "),
        tag_no_case("dword ptr "),
        tag_no_case("word ptr "),
        tag_no_case("byte ptr "),
    )))
    .parse(input)?;
    let size = match size_str.map(|s| s.to_lowercase()) {
        Some(s) if s.starts_with("qword") => MemorySize::Qword,
        Some(s) if s.starts_with("dword") => MemorySize::Dword,
        Some(s) if s.starts_with("word") => MemorySize::Word,
        Some(s) if s.starts_with("byte") => MemorySize::Byte,
        _ => MemorySize::Unspecified,
    };

    let (mut curr, _) = (char('['), sp).parse(input)?;

    let mut base = None;
    let mut index = None;
    let mut scale = 1;
    let mut disp = 0;
    let mut first = true;

    loop {
        let (rest, _) = sp(curr)?;
        if let Ok((rest2, _)) = char::<&str, nom::error::Error<&str>>(']').parse(rest) {
            curr = rest2;
            break;
        }

        let (rest, sign) = if first {
            opt(alt((char('+'), char('-')))).parse(rest)?
        } else {
            let (r, s) = alt((char('+'), char('-'))).parse(rest)?;
            (r, Some(s))
        };

        let (rest, _) = sp(rest)?;
        let is_neg = sign == Some('-');
        let (rest, term) = mem_term(rest)?;
        curr = rest;

        match term {
            MemTerm::Reg(r) => {
                if base.is_none() {
                    base = Some(r);
                } else if index.is_none() {
                    index = Some(r);
                } else {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        curr,
                        nom::error::ErrorKind::Tag,
                    )));
                }
            }
            MemTerm::Scale(r, s) => {
                if index.is_none() {
                    index = Some(r);
                    scale = s;
                } else {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        curr,
                        nom::error::ErrorKind::Tag,
                    )));
                }
            }
            MemTerm::Disp(d) => {
                disp += if is_neg { -d } else { d };
            }
        }
        first = false;
    }

    Ok((
        curr,
        Operand::Memory {
            size,
            base,
            index,
            scale,
            disp,
        },
    ))
}

fn operand(input: &str) -> IResult<&str, Operand> {
    alt((
        memory,
        map(register, Operand::Reg),
        map(immediate, Operand::Imm),
        map(label_name, Operand::Label),
    ))
    .parse(input)
}

fn parse_cond(s: &str) -> Option<Condition> {
    match s {
        "o" => Some(Condition::O),
        "no" => Some(Condition::No),
        "b" | "c" | "nae" => Some(Condition::B),
        "ae" | "nb" | "nc" => Some(Condition::Ae),
        "e" | "z" => Some(Condition::E),
        "ne" | "nz" => Some(Condition::Ne),
        "be" | "na" => Some(Condition::Be),
        "a" | "nbe" => Some(Condition::A),
        "s" => Some(Condition::S),
        "ns" => Some(Condition::Ns),
        "p" | "pe" => Some(Condition::P),
        "np" | "po" => Some(Condition::Np),
        "l" | "nge" => Some(Condition::L),
        "ge" | "nl" => Some(Condition::Ge),
        "le" | "ng" => Some(Condition::Le),
        "g" | "nle" => Some(Condition::G),
        _ => None,
    }
}

fn mnemonic_parser(input: &str) -> IResult<&str, Mnemonic> {
    let (rest, token) = take_while1(|c: char| c.is_alphabetic())(input)?;
    let lower = token.to_lowercase();

    if let Some(cond_str) = lower.strip_prefix("j") {
        if cond_str == "mp" {
            return Ok((rest, Mnemonic::Jmp));
        }
        if let Some(cond) = parse_cond(cond_str) {
            return Ok((rest, Mnemonic::Jcc(cond)));
        }
    }
    if let Some(cond_str) = lower.strip_prefix("cmov") {
        if let Some(cond) = parse_cond(cond_str) {
            return Ok((rest, Mnemonic::Cmov(cond)));
        }
    }
    if let Some(cond_str) = lower.strip_prefix("set") {
        if let Some(cond) = parse_cond(cond_str) {
            return Ok((rest, Mnemonic::Set(cond)));
        }
    }

    let mnem = match lower.as_str() {
        "mov" => Mnemonic::Mov,
        "add" => Mnemonic::Add,
        "sub" => Mnemonic::Sub,
        "cmp" => Mnemonic::Cmp,
        "test" => Mnemonic::Test,
        "and" => Mnemonic::And,
        "or" => Mnemonic::Or,
        "xor" => Mnemonic::Xor,
        "lea" => Mnemonic::Lea,
        "mul" => Mnemonic::Mul,
        "div" => Mnemonic::Div,
        "inc" => Mnemonic::Inc,
        "dec" => Mnemonic::Dec,
        "call" => Mnemonic::Call,
        "push" => Mnemonic::Push,
        "pop" => Mnemonic::Pop,
        "nop" => Mnemonic::Nop,
        "ret" => Mnemonic::Ret,
        _ => {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }
    };
    Ok((rest, mnem))
}

pub fn parse_statement(input: &str, line: usize) -> Result<Statement, AsmError> {
    let (rest, _) = sp(input).unwrap();
    if rest.is_empty() {
        return Ok(Statement {
            label: None,
            mnemonic: Mnemonic::LabelOnly,
            operands: vec![],
            line,
        });
    }

    let (rest, label) = match opt(terminated(label_name, (sp, char(':'), sp))).parse(rest) {
        Ok(res) => res,
        Err(_) => (rest, None),
    };

    let (rest, _) = sp(rest).unwrap();
    if rest.is_empty() {
        return Ok(Statement {
            label,
            mnemonic: Mnemonic::LabelOnly,
            operands: vec![],
            line,
        });
    }

    if let Ok((dir_rest, _)) = char::<&str, nom::error::Error<&str>>('.').parse(rest) {
        let (dir_rest, dir_name) =
            take_while1::<_, _, nom::error::Error<&str>>(|c: char| c.is_alphabetic())(dir_rest)
                .unwrap();
        let (dir_rest, _) = sp(dir_rest).unwrap();

        let mnem = match dir_name {
            "byte" => Mnemonic::Byte,
            "short" | "word" => Mnemonic::Short,
            "int" | "long" | "dword" => Mnemonic::Word,
            "space" | "skip" => Mnemonic::Space,
            "text" => Mnemonic::Text,
            "data" => Mnemonic::Data,
            "global" => Mnemonic::Global,
            _ => {
                return Err(AsmError::UnknownMnemonic {
                    line,
                    mnemonic: dir_name.to_string(),
                });
            }
        };

        let mut operands = vec![];
        let mut curr = dir_rest;

        if mnem == Mnemonic::Global {
            if let Ok((next, lbl)) = label_name(curr) {
                operands.push(Operand::Label(lbl));
                curr = next;
            }
        } else if mnem != Mnemonic::Text && mnem != Mnemonic::Data {
            let (next, op_opt) = opt(immediate).parse(curr).unwrap();
            if let Some(imm) = op_opt {
                operands.push(Operand::Expr(Expr::Number(imm)));
            }
            curr = next;
        }

        let (curr, _) = sp(curr).unwrap();
        if !curr.is_empty() {
            return Err(AsmError::ParseError {
                line,
                col: 0,
                message: format!("Unexpected trailing characters: '{}'", curr),
            });
        }
        return Ok(Statement {
            label,
            mnemonic: mnem,
            operands,
            line,
        });
    }

    let (rest, mnemonic) = mnemonic_parser(rest).map_err(|_| AsmError::UnknownMnemonic {
        line,
        mnemonic: rest.to_string(),
    })?;
    let (rest, _) = sp(rest).unwrap();

    let mut operands = vec![];
    let mut curr = rest;
    if !curr.is_empty() {
        let (next, op1) = operand(curr).map_err(|_| AsmError::ParseError {
            line,
            col: 0,
            message: "Invalid operand".into(),
        })?;
        operands.push(op1);
        curr = next;

        if let Ok((next2, _)) = (sp, char(','), sp).parse(curr) {
            let (next3, op2) = operand(next2).map_err(|_| AsmError::ParseError {
                line,
                col: 0,
                message: "Invalid second operand".into(),
            })?;
            operands.push(op2);
            curr = next3;
        }
    }

    let (curr, _) = sp(curr).unwrap();
    if !curr.is_empty() {
        return Err(AsmError::ParseError {
            line,
            col: 0,
            message: format!("Unexpected trailing characters: '{}'", curr),
        });
    }
    Ok(Statement {
        label,
        mnemonic,
        operands,
        line,
    })
}
