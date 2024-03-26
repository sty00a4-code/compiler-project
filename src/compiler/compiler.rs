use super::bytecode::{Closure, Register};
use crate::{lexer::position::Located, parser::ast::*};
use std::collections::HashMap;

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
    fn compile(self, compiler: &mut Compiler) -> Self::Output;
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
    pub fn local(&self, ident: &str) -> Option<Register> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.locals.get(ident).copied())
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

impl Compilable for Located<Chunk> {
    type Output = ();
    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        compiler.push_frame();
        for stat in self.value.0 {
            stat.compile(compiler);
        }
        compiler.pop_frame();
    }
}
impl Compilable for Located<Block> {
    type Output = ();
    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        compiler.frame_mut().push_scope();
        for stat in self.value.0 {
            stat.compile(compiler);
        }
        compiler.frame_mut().pop_scope();
    }
}
impl Compilable for Located<Statement> {
    type Output = Option<Register>;
    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        todo!()
    }
}
impl Compilable for Located<Expression> {
    type Output = Register;
    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        todo!()
    }
}
impl Compilable for Located<Atom> {
    type Output = Register;
    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        todo!()
    }
}
