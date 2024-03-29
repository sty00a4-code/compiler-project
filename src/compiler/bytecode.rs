use std::rc::Rc;
use crate::lexer::position::{Located, Position};

pub type Register = u16;
pub type Address = u32;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ByteCode {
    #[default]
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
        offset: Register,
        args_len: u8,
        dst: Option<Register>,
    },
    Return {
        src: Option<Register>,
    },

    Move {
        dst: Register,
        src: Register,
    },
    String {
        dst: Register,
        addr: Address,
    },
    Number {
        dst: Register,
        addr: Address,
    },
    Closure {
        dst: Register,
        addr: Address,
    },

    Global {
        dst: Register,
        addr: Address,
    },
    SetGlobal {
        addr: Address,
        src: Register
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
    pub closures: Vec<Rc<Self>>,
}

impl Closure {
    pub fn string(&self, addr: Address) -> Option<&String> {
        self.strings.get(addr as usize)
    }
    pub fn number(&self, addr: Address) -> Option<&f64> {
        self.numbers.get(addr as usize)
    }
    pub fn closure(&self, addr: Address) -> Option<&Rc<Closure>> {
        self.closures.get(addr as usize)
    }
    pub fn new_string(&mut self, string: String) -> Address {
        if let Some(addr) = self.strings.iter().position(|s| s == &string) {
            return addr as Address
        }
        let addr = self.strings.len();
        self.strings.push(string);
        addr as Address
    }
    pub fn new_closure(&mut self, closure: Rc<Closure>) -> Address {
        let addr = self.closures.len();
        self.closures.push(closure);
        addr as Address
    }
    pub fn new_number(&mut self, number: f64) -> Address {
        if let Some(addr) = self.numbers.iter().position(|n| n == &number) {
            return addr as Address
        }
        let addr = self.numbers.len();
        self.numbers.push(number);
        addr as Address
    }
    pub fn write(&mut self, bytecode: ByteCode, pos: Position) -> Address {
        let addr = self.code.len();
        self.code.push(Located::new(bytecode, pos));
        addr as Address
    }
    pub fn overwrite(&mut self, addr: Address, bytecode: ByteCode) {
        let Located {
            value: old_bytecode,
            pos: _,
        } = self
            .code
            .get_mut(addr as usize)
            .expect("invalid overwrite address");
        *old_bytecode = bytecode;
    }
}
