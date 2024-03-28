use super::interpreter::Interpreter;
use crate::compiler::bytecode::Closure;
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
    Boolean(bool),
    String(Rc<str>),
    Function(Rc<Function>),
}
#[derive(Debug, Clone, PartialEq)]
pub enum Function {
    NativeFunction(NativeFunction),
    Function(Rc<Closure>),
}
pub type NativeFunction = fn(&mut Interpreter, Vec<Value>) -> Result<Value, Box<dyn Error>>;

impl Value {
    pub fn typ(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Number(_) => "number",
            Value::Boolean(_) => "boolean",
            Value::String(_) => "string",
            Value::Function(_) => "function",
        }
    }
}

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
            Value::Boolean(bool) => write!(f, "{bool:?}"),
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
        Self::String(value.into())
    }
}
impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.into())
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

impl From<&Value> for bool {
    fn from(value: &Value) -> bool {
        match value {
            Value::Null => false,
            Value::Number(v) => *v == 0.,
            Value::Boolean(v) => *v,
            Value::String(v) => !v.is_empty(),
            Value::Function(_) => true,
        }
    }
}
