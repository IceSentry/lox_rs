use crate::{interpreter::Interpreter, lox::LoxValue};
use std::fmt::{self, Display, Formatter};

#[derive(Clone)]
pub enum Function {
    Native(usize, Box<fn(&Vec<LoxValue>) -> LoxValue>),
}

impl Function {
    pub fn arity(&self) -> usize {
        match self {
            Function::Native(arity, _) => *arity,
        }
    }
    pub fn call(&self, _interpreter: &mut Interpreter, args: &Vec<LoxValue>) -> LoxValue {
        match self {
            Function::Native(_, body) => body(args),
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Function::Native(_, _) => write!(f, "<native fn>"),
        }
    }
}
