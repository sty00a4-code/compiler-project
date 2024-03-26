use crate::lexer::position::Located;

pub type Register = u16;
pub type Address = u32;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ByteCode {
    None,
    Jump {
        addr: Address,
    },
    JumpIf {
        not: bool,
        cond: Register,
        addr: Address,
    },

    Call {
        func: Register,
        args: Register,
        args_len: u8,
        dst: Option<Register>
    },
    Return {
        src: Option<Register>
    },

    Move {
        dst: Register,
        src: Register,
    },
    String {
        dst: Register,
        addr: Address
    },
    Number {
        dst: Register,
        addr: Address
    },
    Global {
        dst: Register,
        addr: Address
    },

    Binary {
        op: BinaryOperation,
        dst: Register,
        left: Register,
        right: Register,
    },
    Unary {
        op: UnaryOperation,
        dst: Register,
        src: Register,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BinaryOperation {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    EQ,
    NE,
    LT,
    GT,
    LE,
    GE,
    And,
    Or,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum UnaryOperation {
    Neg,
    Not,
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Closure {
    pub code: Vec<Located<ByteCode>>,
    pub registers: Register,
    pub strings: Vec<String>,
    pub numbers: Vec<f64>,
}

impl Closure {
    pub fn string(&self, addr: Address) -> Option<&String> {
        self.strings.get(addr as usize)
    }
    pub fn number(&self, addr: Address) -> Option<&f64> {
        self.numbers.get(addr as usize)
    }
}