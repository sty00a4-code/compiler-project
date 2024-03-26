use super::interpreter::Interpreter;
use crate::{compiler::bytecode::Closure, lexer::position::Located};
use std::{
    error::Error,
    fmt::{Debug, Display},
    rc::Rc,
};

#[derive(Clone, PartialEq, Default)]
pub enum Value {
    #[default]
    Null,
    Number(f64),
    String(String),
    Function(Rc<Function>),
}
#[derive(Debug, Clone, PartialEq)]
pub enum Function {
    NativeFunction(NativeFunction),
    Function(Rc<Closure>),
}
pub type NativeFunction =
    fn(&mut Interpreter, Vec<Value>) -> Result<Value, Located<Box<dyn Error>>>;

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(number) => write!(f, "{number}"),
            Value::String(string) => write!(f, "{string}"),
            _ => write!(f, "{self:?}"),
        }
    }
}
impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Number(number) => write!(f, "{number:?}"),
            Value::String(string) => write!(f, "{string:?}"),
            Value::Function(func) => write!(f, "function:{:08x?}", Rc::as_ptr(func)),
        }
    }
}
impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}
impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}
impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}
impl From<NativeFunction> for Value {
    fn from(value: NativeFunction) -> Self {
        Self::Function(Rc::new(Function::NativeFunction(value)))
    }
}
impl From<Closure> for Value {
    fn from(value: Closure) -> Self {
        Self::Function(Rc::new(Function::Function(Rc::new(value))))
    }
}
