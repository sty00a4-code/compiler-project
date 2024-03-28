use super::value::{Function, NativeFunction, Value};
use crate::{
    compiler::bytecode::{Address, BinaryOperation, ByteCode, Closure, Register, UnaryOperation},
    lexer::position::{Located, Position},
};
use std::{cell::RefCell, collections::HashMap, error::Error, fmt::Display, rc::Rc};

#[derive(Debug, Default)]
pub struct Interpreter {
    pub call_stack: Vec<CallFrame>,
    pub globals: HashMap<String, Value>,
}
#[derive(Debug, Clone)]
pub struct CallFrame {
    pub closure: Rc<Closure>,
    pub ip: Address,
    pub stack: Vec<Rc<RefCell<Value>>>,
    pub dst: Option<Register>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RunTimeError {
    Binary {
        left: &'static str,
        right: &'static str,
    },
    Unary {
        right: &'static str,
    },
    CannotCall(&'static str),
    Custome(String),
}
impl Display for RunTimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunTimeError::Binary { left, right } => {
                write!(f, "cannot perform binary operation on {left} with {right}")
            }
            RunTimeError::Unary { right } => {
                write!(f, "cannot perform binary operation on {right}")
            }
            RunTimeError::CannotCall(typ) => write!(f, "cannot call {typ}"),
            RunTimeError::Custome(err) => write!(f, "{err}"),
        }
    }
}
impl CallFrame {
    pub fn instr(&self) -> Option<&Located<ByteCode>> {
        self.closure.code.get(self.ip as usize)
    }
    pub fn register(&self, register: Register) -> Option<&Rc<RefCell<Value>>> {
        self.stack.get(register as usize)
    }
}
impl Interpreter {
    pub fn call_frame(&self) -> Option<&CallFrame> {
        self.call_stack.last()
    }
    pub fn call_frame_mut(&mut self) -> Option<&mut CallFrame> {
        self.call_stack.last_mut()
    }
    pub fn register(&mut self, register: Register) -> Option<&Rc<RefCell<Value>>> {
        self.call_frame_mut()
            .expect("no call frame on stack")
            .register(register)
    }
    pub fn call_closure(&mut self, closure: &Rc<Closure>, args: Vec<Value>, dst: Option<Register>) {
        let mut stack = Vec::with_capacity(closure.registers as usize + 1);
        let args_len = args.len();
        stack.extend(args.into_iter().map(|v| Rc::new(RefCell::new(v))));
        stack.extend(
            (args_len..=closure.registers as usize).map(|_| Rc::new(RefCell::new(Value::default()))),
        );
        self.call_stack.push(CallFrame {
            closure: Rc::clone(closure),
            ip: 0,
            stack,
            dst,
        });
    }
    pub fn call_native(
        &mut self,
        func: &NativeFunction,
        args: Vec<Value>,
        pos: &Position,
    ) -> Result<Value, Located<Box<dyn Error>>> {
        func(self, args).map_err(|err| Located::new(err, pos.clone()))
    }
    pub fn call(
        &mut self,
        function: &Function,
        args: Vec<Value>,
        pos: &Position,
        dst: Option<Register>,
    ) -> Result<(), Located<Box<dyn Error>>> {
        match function {
            Function::NativeFunction(func) => {
                let value = self.call_native(func, args, pos)?;
                if let Some(dst) = dst {
                    let mut dst = self.register(dst).expect("location not found").borrow_mut();
                    *dst = value;
                }
                Ok(())
            }
            Function::Function(closure) => {
                self.call_closure(closure, args, dst);
                Ok(())
            }
        }
    }
    pub fn return_call(&mut self, src: Option<Register>) -> Option<Value> {
        let top_frame = self.call_stack.pop().expect("no frame on stack");
        if let Some(prev_frame) = self.call_stack.last_mut() {
            if let Some(dst) = top_frame.dst {
                let value = if let Some(src) = src {
                    top_frame
                        .register(src)
                        .expect("source not found")
                        .borrow()
                        .clone()
                } else {
                    Value::default()
                };
                let dst = prev_frame.register(dst).expect("location not found");
                *dst.borrow_mut() = value;
            }
        }
        if let Some(src) = src {
            return Some(
                top_frame
                    .register(src)
                    .expect("source not found")
                    .borrow()
                    .clone(),
            );
        }
        None
    }
    pub fn step(&mut self) -> Result<Option<Value>, Located<RunTimeError>> {
        let Located {
            value: bytecode,
            pos,
        } = self
            .call_frame()
            .expect("no call frame on stack")
            .instr()
            .expect("ip out of range")
            .clone();
        self.call_frame_mut().expect("no call frame on stack").ip += 1;
        match bytecode {
            ByteCode::None => {}
            ByteCode::Jump { addr } => {
                self.call_frame_mut().expect("no call frame on stack").ip = addr;
            }
            ByteCode::JumpIf {
                not: false,
                cond,
                addr,
            } => {
                let cond = self
                    .register(cond)
                    .expect("register not found")
                    .borrow()
                    .clone();
                if bool::from(&cond) {
                    self.call_frame_mut().expect("no call frame on stack").ip = addr;
                }
            }
            ByteCode::JumpIf {
                not: true,
                cond,
                addr,
            } => {
                let cond = self
                    .register(cond)
                    .expect("register not found")
                    .borrow()
                    .clone();
                if !bool::from(&cond) {
                    self.call_frame_mut().expect("no call frame on stack").ip = addr;
                }
            }
            ByteCode::Call {
                func,
                offset,
                args_len,
                dst,
            } => {
                let func = self
                    .register(func)
                    .expect("register not found")
                    .borrow()
                    .clone();
                let mut args: Vec<Value> = Vec::with_capacity(args_len as usize);
                args.extend((offset..offset + args_len as Register).map(|reg| {
                    self.register(reg)
                        .expect("register not found")
                        .borrow()
                        .clone()
                }));
                match func {
                    Value::Function(function) => {
                        self.call(&function, args, &pos, dst)
                            .map_err(|err| err.map(|err| RunTimeError::Custome(err.to_string())))?;
                    }
                    value => return Err(Located::new(RunTimeError::CannotCall(value.typ()), pos)),
                }
            }
            ByteCode::Return { src } => return Ok(self.return_call(src)),
            ByteCode::Move { dst, src } => {
                let src = self
                    .register(src)
                    .expect("register not found")
                    .borrow()
                    .clone();
                let mut dst = self.register(dst).expect("register not found").borrow_mut();
                *dst = src;
            }
            ByteCode::String { dst, addr } => {
                let string = self
                    .call_frame()
                    .expect("no call frame on stack")
                    .closure
                    .string(addr)
                    .expect("string not found")
                    .clone();
                let mut dst = self.register(dst).expect("register not found").borrow_mut();
                *dst = Value::String(string.into());
            }
            ByteCode::Number { dst, addr } => {
                let number = self
                    .call_frame()
                    .expect("no call frame on stack")
                    .closure
                    .number(addr)
                    .expect("string not found")
                    .clone();
                let mut dst = self.register(dst).expect("register not found").borrow_mut();
                *dst = Value::Number(number);
            }
            ByteCode::Closure { dst, addr } => {
                let closure = Rc::clone(
                    self.call_frame()
                        .expect("no call frame on stack")
                        .closure
                        .closure(addr)
                        .expect("string not found"),
                );
                let mut dst = self.register(dst).expect("register not found").borrow_mut();
                *dst = Value::Function(Rc::new(Function::Function(closure)));
            }
            ByteCode::Global { dst, addr } => {
                let value = {
                    let string = self
                        .call_frame()
                        .expect("no call frame on stack")
                        .closure
                        .string(addr)
                        .expect("string not found");
                    self.globals.get(string).cloned().unwrap_or_default()
                };
                let mut dst = self.register(dst).expect("register not found").borrow_mut();
                *dst = value;
            }
            ByteCode::Binary {
                op,
                dst,
                left,
                right,
            } => {
                let left = self
                    .register(left)
                    .expect("register not found")
                    .borrow()
                    .clone();
                let right = self
                    .register(right)
                    .expect("register not found")
                    .borrow()
                    .clone();
                let mut dst = self.register(dst).expect("register not found").borrow_mut();
                *dst = match op {
                    BinaryOperation::Add => match (left, right) {
                        (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
                        (left, right) => {
                            return Err(Located::new(
                                RunTimeError::Binary {
                                    left: left.typ(),
                                    right: right.typ(),
                                },
                                pos,
                            ))
                        }
                    },
                    BinaryOperation::Sub => match (left, right) {
                        (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
                        (left, right) => {
                            return Err(Located::new(
                                RunTimeError::Binary {
                                    left: left.typ(),
                                    right: right.typ(),
                                },
                                pos,
                            ))
                        }
                    },
                    BinaryOperation::Mul => match (left, right) {
                        (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
                        (left, right) => {
                            return Err(Located::new(
                                RunTimeError::Binary {
                                    left: left.typ(),
                                    right: right.typ(),
                                },
                                pos,
                            ))
                        }
                    },
                    BinaryOperation::Div => match (left, right) {
                        (Value::Number(a), Value::Number(b)) => Value::Number(a / b),
                        (left, right) => {
                            return Err(Located::new(
                                RunTimeError::Binary {
                                    left: left.typ(),
                                    right: right.typ(),
                                },
                                pos,
                            ))
                        }
                    },
                    BinaryOperation::Mod => match (left, right) {
                        (Value::Number(a), Value::Number(b)) => Value::Number(a % b),
                        (left, right) => {
                            return Err(Located::new(
                                RunTimeError::Binary {
                                    left: left.typ(),
                                    right: right.typ(),
                                },
                                pos,
                            ))
                        }
                    },
                    BinaryOperation::Pow => match (left, right) {
                        (Value::Number(a), Value::Number(b)) => Value::Number(a.powf(b)),
                        (left, right) => {
                            return Err(Located::new(
                                RunTimeError::Binary {
                                    left: left.typ(),
                                    right: right.typ(),
                                },
                                pos,
                            ))
                        }
                    },
                    BinaryOperation::EQ => Value::Boolean(left == right),
                    BinaryOperation::NE => Value::Boolean(left != right),
                    BinaryOperation::LT => match (left, right) {
                        (Value::Number(a), Value::Number(b)) => Value::Boolean(a < b),
                        (left, right) => {
                            return Err(Located::new(
                                RunTimeError::Binary {
                                    left: left.typ(),
                                    right: right.typ(),
                                },
                                pos,
                            ))
                        }
                    },
                    BinaryOperation::GT => match (left, right) {
                        (Value::Number(a), Value::Number(b)) => Value::Boolean(a > b),
                        (left, right) => {
                            return Err(Located::new(
                                RunTimeError::Binary {
                                    left: left.typ(),
                                    right: right.typ(),
                                },
                                pos,
                            ))
                        }
                    },
                    BinaryOperation::LE => match (left, right) {
                        (Value::Number(a), Value::Number(b)) => Value::Boolean(a <= b),
                        (left, right) => {
                            return Err(Located::new(
                                RunTimeError::Binary {
                                    left: left.typ(),
                                    right: right.typ(),
                                },
                                pos,
                            ))
                        }
                    },
                    BinaryOperation::GE => match (left, right) {
                        (Value::Number(a), Value::Number(b)) => Value::Boolean(a >= b),
                        (left, right) => {
                            return Err(Located::new(
                                RunTimeError::Binary {
                                    left: left.typ(),
                                    right: right.typ(),
                                },
                                pos,
                            ))
                        }
                    },
                    BinaryOperation::And => Value::Boolean(bool::from(&left) && bool::from(&right)),
                    BinaryOperation::Or => Value::Boolean(bool::from(&left) || bool::from(&right)),
                };
            }
            ByteCode::Unary { op, dst, src } => {
                let right = self
                    .register(src)
                    .expect("register not found")
                    .borrow()
                    .clone();
                let mut dst = self.register(dst).expect("register not found").borrow_mut();
                *dst = match op {
                    UnaryOperation::Neg => match right {
                        Value::Number(v) => Value::Number(-v),
                        right => {
                            return Err(Located::new(
                                RunTimeError::Unary { right: right.typ() },
                                pos,
                            ))
                        }
                    },
                    UnaryOperation::Not => Value::Boolean(!bool::from(&right)),
                }
            }
        }
        Ok(None)
    }
    pub fn run(&mut self, closure: &Rc<Closure>) -> Result<Option<Value>, Located<RunTimeError>> {
        let offset = self.call_stack.len();
        self.call_closure(closure, vec![], None);
        loop {
            let value = self.step()?;
            if self.call_stack.len() <= offset {
                return Ok(value);
            }
        }
    }
}
