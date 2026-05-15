use crate::error::AsmError;
use crate::types::*;
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag_no_case, take_while, take_while1},
    character::complete::{char, digit1, hex_digit1, space0},
    combinator::{map, map_res, opt, recognize, value},
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
        "rax" => Reg::Rax,
        "rcx" => Reg::Rcx,
        "rdx" => Reg::Rdx,
        "rbx" => Reg::Rbx,
        "rsp" => Reg::Rsp,
        "rbp" => Reg::Rbp,
        "rsi" => Reg::Rsi,
        "rdi" => Reg::Rdi,
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
    let (input, _) = opt(alt((
        tag_no_case("qword ptr "),
        tag_no_case("dword ptr "),
        tag_no_case("word ptr "),
        tag_no_case("byte ptr "),
    )))
    .parse(input)?;
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

fn mnemonic_parser(input: &str) -> IResult<&str, Mnemonic> {
    alt((
        value(Mnemonic::Mov, tag_no_case("mov")),
        value(Mnemonic::Add, tag_no_case("add")),
        value(Mnemonic::Sub, tag_no_case("sub")),
        value(Mnemonic::Cmp, tag_no_case("cmp")),
        value(Mnemonic::And, tag_no_case("and")),
        value(Mnemonic::Or, tag_no_case("or")),
        value(Mnemonic::Xor, tag_no_case("xor")),
        value(Mnemonic::Jmp, tag_no_case("jmp")),
        value(Mnemonic::Call, tag_no_case("call")),
        value(Mnemonic::Push, tag_no_case("push")),
        value(Mnemonic::Pop, tag_no_case("pop")),
        value(Mnemonic::Nop, tag_no_case("nop")),
        value(Mnemonic::Ret, tag_no_case("ret")),
        value(Mnemonic::Lea, tag_no_case("lea")),
        value(Mnemonic::Mul, tag_no_case("mul")),
        value(Mnemonic::Div, tag_no_case("div")),
        value(Mnemonic::Inc, tag_no_case("inc")),
        value(Mnemonic::Dec, tag_no_case("dec")),
    ))
    .parse(input)
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

    // Directives Parsing
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

        // Parse operands depending on the directive type
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

    // STRICT CHECK: Ensure the line was fully consumed
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
