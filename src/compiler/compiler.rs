use super::bytecode::{Address, BinaryOperation, ByteCode, Closure, Register, UnaryOperation};
use crate::{
    lexer::position::{Located, Position},
    parser::ast::*,
};
use std::{collections::HashMap, rc::Rc};

#[derive(Debug, Default)]
pub struct Compiler {
    pub frames: Vec<Frame>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub closure: Closure,
    pub registers: Register,
    pub scopes: Vec<Scope>,
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Scope {
    pub locals: HashMap<String, Register>,
    pub offset: Register,
}

pub trait Compilable {
    type Output;
    type Error;
    fn compile(self, compiler: &mut Compiler) -> Result<Self::Output, Self::Error>;
}
impl Compiler {
    pub fn push_frame(&mut self) {
        self.frames.push(Frame {
            closure: Closure::default(),
            registers: 0,
            scopes: vec![Scope::default()],
        })
    }
    pub fn pop_frame(&mut self) -> Option<Frame> {
        self.frames.pop()
    }
    pub fn frame(&self) -> &Frame {
        self.frames.last().unwrap()
    }
    pub fn frame_mut(&mut self) -> &mut Frame {
        self.frames.last_mut().unwrap()
    }
}
impl Frame {
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope {
            offset: self.registers,
            ..Default::default()
        });
    }
    pub fn pop_scope(&mut self) {
        if let Some(scope) = self.scopes.pop() {
            self.registers = scope.offset;
        }
    }
    pub fn local(&mut self, ident: &str, pos: Position) -> Register {
        if let Some(reg) = self
            .scopes
            .iter()
            .rev()
            .find_map(|scope| scope.locals.get(ident).copied())
        {
            reg
        } else {
            let reg = self.new_register();
            let addr = self.closure.new_string(ident.to_string());
            self.closure.write(ByteCode::Global { dst: reg, addr }, pos);
            reg
        }
    }
    pub fn new_register(&mut self) -> Register {
        let register = self.registers;
        self.registers += 1;
        if self.closure.registers < self.registers {
            self.closure.registers = self.registers;
        }
        register
    }
    pub fn new_local(&mut self, ident: String) -> Register {
        let register = self.new_register();
        self.scopes
            .last_mut()
            .unwrap()
            .locals
            .insert(ident, register);
        register
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompileError {}
impl Compilable for Located<Chunk> {
    type Output = Closure;
    type Error = Located<CompileError>;
    fn compile(self, compiler: &mut Compiler) -> Result<Self::Output, Self::Error> {
        compiler.push_frame();
        for stat in self.value.0 {
            stat.compile(compiler)?;
        }
        compiler.frame_mut().closure.write(ByteCode::Return { src: None }, self.pos);
        let frame = compiler.pop_frame().unwrap();
        Ok(frame.closure)
    }
}
impl Compilable for Located<Block> {
    type Output = Option<u16>;
    type Error = Located<CompileError>;
    fn compile(self, compiler: &mut Compiler) -> Result<Self::Output, Self::Error> {
        compiler.frame_mut().push_scope();
        for stat in self.value.0 {
            if let Some(reg) = stat.compile(compiler)? {
                compiler.frame_mut().pop_scope();
                return Ok(Some(reg));
            }
        }
        compiler.frame_mut().pop_scope();
        Ok(None)
    }
}
impl Compilable for Located<Statement> {
    type Output = Option<Register>;
    type Error = Located<CompileError>;
    fn compile(self, compiler: &mut Compiler) -> Result<Self::Output, Self::Error> {
        let Located { value: stat, pos } = self;
        match stat {
            Statement::Block(block) => Located::new(block, pos).compile(compiler),
            Statement::Let {
                ident:
                    Located {
                        value: ident,
                        pos: _,
                    },
                expr,
            } => {
                let reg = compiler.frame_mut().new_local(ident);
                let src = expr.compile(compiler)?;
                compiler
                    .frame_mut()
                    .closure
                    .write(ByteCode::Move { dst: reg, src }, pos);
                Ok(None)
            }
            Statement::Assign {
                ident:
                    Located {
                        value: ident,
                        pos: ident_pos,
                    },
                expr,
            } => {
                let reg = compiler.frame_mut().local(&ident, ident_pos);
                let src = expr.compile(compiler)?;
                compiler
                    .frame_mut()
                    .closure
                    .write(ByteCode::Move { dst: reg, src }, pos);
                Ok(None)
            }
            Statement::Call {
                ident:
                    Located {
                        value: ident,
                        pos: ident_pos,
                    },
                args,
            } => {
                let func = compiler.frame_mut().local(&ident, ident_pos);
                let args_len = args.len() as u8;
                let offset = compiler.frame().registers;
                compiler.frame_mut().registers += args_len as Register;
                for (reg, arg) in (offset..offset + args_len as Register).zip(args) {
                    let pos = arg.pos.clone();
                    let src = arg.compile(compiler)?;
                    compiler
                        .frame_mut()
                        .closure
                        .write(ByteCode::Move { dst: reg, src }, pos);
                }
                compiler.frame_mut().closure.write(
                    ByteCode::Call {
                        func,
                        offset,
                        args_len,
                        dst: None,
                    },
                    pos,
                );
                Ok(None)
            }
            Statement::Def {
                ident:
                    Located {
                        value: ident,
                        pos: _,
                    },
                params,
                body,
            } => {
                let reg = compiler.frame_mut().new_local(ident);
                let addr = {
                    compiler.push_frame();
                    for Located {
                        value: param,
                        pos: _,
                    } in params
                    {
                        compiler.frame_mut().new_local(param);
                    }
                    body.compile(compiler)?;
                    compiler.frame_mut().closure.write(ByteCode::Return { src: None }, pos.clone());
                    let frame = compiler.pop_frame().unwrap();
                    let closure = Rc::new(frame.closure);
                    compiler.frame_mut().closure.new_closure(closure)
                };
                compiler
                    .frame_mut()
                    .closure
                    .write(ByteCode::Closure { dst: reg, addr }, pos);
                Ok(None)
            }
            Statement::If {
                cond,
                case,
                else_case,
            } => {
                let cond = cond.compile(compiler)?;
                let check_addr = compiler
                    .frame_mut()
                    .closure
                    .write(ByteCode::default(), pos.clone());
                case.compile(compiler)?;
                let case_exit_addr = compiler.frame_mut().closure.write(ByteCode::default(), pos);
                let else_addr = compiler.frame_mut().closure.code.len() as Address;
                if let Some(else_case) = else_case {
                    else_case.compile(compiler)?;
                }
                let exit_addr = compiler.frame_mut().closure.code.len() as Address;
                compiler.frame_mut().closure.overwrite(
                    check_addr,
                    ByteCode::JumpIf {
                        not: true,
                        cond,
                        addr: else_addr,
                    },
                );
                compiler
                    .frame_mut()
                    .closure
                    .overwrite(case_exit_addr, ByteCode::Jump { addr: exit_addr });
                Ok(None)
            }
            Statement::While { cond, body } => {
                let cond_addr = compiler.frame_mut().closure.code.len() as Address;
                let cond = cond.compile(compiler)?;
                let check_addr = compiler
                    .frame_mut()
                    .closure
                    .write(ByteCode::default(), pos.clone());
                body.compile(compiler)?;
                compiler
                    .frame_mut()
                    .closure
                    .write(ByteCode::Jump { addr: cond_addr }, pos);
                let exit_addr = compiler
                    .frame_mut()
                    .closure
                    .code
                    .len() as Address;
                compiler.frame_mut().closure.overwrite(
                    check_addr,
                    ByteCode::JumpIf {
                        not: true,
                        cond,
                        addr: exit_addr,
                    },
                );
                Ok(None)
            }
            Statement::Return(expr) => {
                let src = expr.compile(compiler)?;
                compiler.frame_mut().closure.write(ByteCode::Return { src: Some(src) }, pos);
                Ok(Some(src))
            }
        }
    }
}
impl From<BinaryOperator> for BinaryOperation {
    fn from(value: BinaryOperator) -> Self {
        match value {
            BinaryOperator::Plus => Self::Add,
            BinaryOperator::Minus => Self::Sub,
            BinaryOperator::Star => Self::Mul,
            BinaryOperator::Slash => Self::Div,
            BinaryOperator::Percent => Self::Mod,
            BinaryOperator::Exponent => Self::Pow,
            BinaryOperator::EqualEqual => Self::EQ,
            BinaryOperator::ExclamationEqual => Self::NE,
            BinaryOperator::Less => Self::LT,
            BinaryOperator::Greater => Self::GT,
            BinaryOperator::LessEqual => Self::LE,
            BinaryOperator::GreaterEqual => Self::GE,
            BinaryOperator::Ampersand => Self::And,
            BinaryOperator::Pipe => Self::Or,
        }
    }
}
impl From<UnaryOperator> for UnaryOperation {
    fn from(value: UnaryOperator) -> Self {
        match value {
            UnaryOperator::Minus => Self::Neg,
            UnaryOperator::Exclamation => Self::Not,
        }
    }
}
impl Compilable for Located<Expression> {
    type Output = Register;
    type Error = Located<CompileError>;
    fn compile(self, compiler: &mut Compiler) -> Result<Self::Output, Self::Error> {
        let Located { value: expr, pos } = self;
        match expr {
            Expression::Atom(atom) => Located::new(atom, pos).compile(compiler),
            Expression::Binary { op, left, right } => {
                let dst = compiler.frame_mut().new_register();
                let left = left.compile(compiler)?;
                let right = right.compile(compiler)?;
                let op = op.into();
                compiler.frame_mut().closure.write(
                    ByteCode::Binary {
                        op,
                        dst,
                        left,
                        right,
                    },
                    pos,
                );
                Ok(dst)
            }
            Expression::Unary { op, right } => {
                let dst = compiler.frame_mut().new_register();
                let src = right.compile(compiler)?;
                let op = op.into();
                compiler
                    .frame_mut()
                    .closure
                    .write(ByteCode::Unary { op, dst, src }, pos);
                Ok(dst)
            }
            Expression::Call { head, args } => {
                let dst = compiler.frame_mut().new_register();
                let func = head.compile(compiler)?;
                let args_len = args.len() as u8;
                let offset = compiler.frame().registers;
                compiler.frame_mut().registers += args_len as Register;
                for (reg, arg) in (offset..offset + args_len as Register).zip(args) {
                    let pos = arg.pos.clone();
                    let src = arg.compile(compiler)?;
                    compiler
                        .frame_mut()
                        .closure
                        .write(ByteCode::Move { dst: reg, src }, pos);
                }
                compiler.frame_mut().closure.write(
                    ByteCode::Call {
                        func,
                        offset,
                        args_len,
                        dst: Some(dst),
                    },
                    pos,
                );
                Ok(dst)
            }
        }
    }
}
impl Compilable for Located<Atom> {
    type Output = Register;
    type Error = Located<CompileError>;
    fn compile(self, compiler: &mut Compiler) -> Result<Self::Output, Self::Error> {
        let Located { value: atom, pos } = self;
        match atom {
            Atom::Ident(ident) => Ok(compiler.frame_mut().local(&ident, pos)),
            Atom::Number(number) => {
                let addr = compiler.frame_mut().closure.new_number(number);
                let dst = compiler.frame_mut().new_register();
                compiler
                    .frame_mut()
                    .closure
                    .write(ByteCode::Number { dst, addr }, pos);
                Ok(dst)
            }
            Atom::String(string) => {
                let addr = compiler.frame_mut().closure.new_string(string);
                let dst = compiler.frame_mut().new_register();
                compiler
                    .frame_mut()
                    .closure
                    .write(ByteCode::String { dst, addr }, pos);
                Ok(dst)
            }
            Atom::Expression(expr) => expr.compile(compiler),
        }
    }
}
