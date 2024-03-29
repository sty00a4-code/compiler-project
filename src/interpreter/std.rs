use super::{
    interpreter::Interpreter,
    value::{Function, Value},
};
use std::{collections::HashMap, error::Error, rc::Rc};

pub fn std_globals(globals: &mut HashMap<String, Value>) {
    globals.insert(
        "print".into(),
        Value::Function(Rc::new(Function::NativeFunction(_print))),
    );
}

fn _print(_: &mut Interpreter, args: Vec<Value>) -> Result<Value, Box<dyn Error>> {
    for arg in args {
        print!("{}", arg);
    }
    println!();
    Ok(Value::default())
}
