use super::value::Value;
use crate::compiler::bytecode::{Closure, Register};
use std::{collections::HashMap, rc::Rc};

pub struct Interpreter {
    pub call_stack: Vec<CallFrame>,
    pub globals: HashMap<String, Value>,
}
#[derive(Debug, Clone)]
pub struct CallFrame {
    pub closure: Rc<Closure>,
    pub stack: Vec<Value>,
    pub dst: Option<Register>,
}
