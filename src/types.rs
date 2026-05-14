#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Reg {
    Al,
    Cl,
    Dl,
    Bl,
    Ax,
    Cx,
    Dx,
    Bx,
    Eax,
    Ecx,
    Edx,
    Ebx,
    Esp,
    Ebp,
    Esi,
    Edi,
    Rax,
    Rcx,
    Rdx,
    Rbx,
    Rsp,
    Rbp,
    Rsi,
    Rdi,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mnemonic {
    Mov,
    Add,
    Sub,
    Cmp,
    And,
    Or,
    Xor,
    Jmp,
    Call,
    Push,
    Pop,
    Nop,
    Ret,
    Mul,
    Div,
    Lea,
    Inc,
    Dec,
    Global,
    Text,
    Data,
    Align,
    Ascii,
    Asciz,
    Word,
    Byte,
    Short,
    Space,
    LabelOnly,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i64),
    Symbol(String),
}

#[derive(Debug, Clone)]
pub enum Operand {
    Reg(Reg),
    Imm(i64),
    Memory { base: Reg, disp: i32 },
    Label(String),
    Expr(Expr),
    StringBytes(Vec<u8>),
}

#[derive(Debug)]
pub struct Statement {
    pub label: Option<String>,
    pub mnemonic: Mnemonic,
    pub operands: Vec<Operand>,
    pub line: usize,
}
