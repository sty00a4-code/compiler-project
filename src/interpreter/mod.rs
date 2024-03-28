use ::std::rc::Rc;

use crate::{compiler::bytecode::Closure, lexer::position::Located};
use self::{interpreter::{Interpreter, RunTimeError}, value::Value, std::std_globals};
pub mod value;
pub mod interpreter;
pub mod std;

pub fn run(closure: &Rc<Closure>) -> Result<Option<Value>, Located<RunTimeError>> {
    let mut interpreter = Interpreter::default();
    std_globals(&mut interpreter.globals);
    interpreter.run(closure)
}