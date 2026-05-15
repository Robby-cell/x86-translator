use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Reg {
    Al,
    Cl,
    Dl,
    Bl,
    Ah,
    Ch,
    Dh,
    Bh,
    Ax,
    Cx,
    Dx,
    Bx,
    Sp,
    Bp,
    Si,
    Di,
    Eax,
    Ecx,
    Edx,
    Ebx,
    Esp,
    Ebp,
    Esi,
    Edi,
    R8d,
    R9d,
    R10d,
    R11d,
    R12d,
    R13d,
    R14d,
    R15d,
    Rax,
    Rcx,
    Rdx,
    Rbx,
    Rsp,
    Rbp,
    Rsi,
    Rdi,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemorySize {
    Unspecified,
    Byte,
    Word,
    Dword,
    Qword,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Condition {
    O,
    No,
    B,
    Ae,
    E,
    Ne,
    Be,
    A,
    S,
    Ns,
    P,
    Np,
    L,
    Ge,
    Le,
    G,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mnemonic {
    Mov,
    Add,
    Adc,
    Sub,
    Sbb,
    Cmp,
    Test,
    And,
    Or,
    Xor,
    Mul,
    Div,
    Lea,
    Inc,
    Dec,
    Jmp,
    Jcc(Condition),
    Cmov(Condition),
    Set(Condition),
    Call,
    Push,
    Pop,
    Nop,
    Ret,
    Hlt,
    Int,
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
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Operand {
    Reg(Reg),
    Imm(i64),
    Memory {
        size: MemorySize,
        base: Option<Reg>,
        index: Option<Reg>,
        scale: u32,
        disp: i32,
    },
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

#[derive(Debug, Clone)]
pub struct AssembleResult {
    pub bytes: Vec<u8>,
    pub entry_point: u64,
    pub labels: HashMap<String, u64>,
    pub instruction_count: usize,
}
