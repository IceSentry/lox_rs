use crate::{interpreter::Interpreter, lox::LoxValue};
use std::fmt::{self, Display, Formatter};

#[derive(Clone)]
pub enum Function {
    Native(usize, Box<fn(&[LoxValue]) -> LoxValue>),
}

impl Function {
    pub fn arity(&self) -> usize {
        match self {
            Function::Native(arity, _) => *arity,
        }
    }
    pub fn call(&self, _interpreter: &mut Interpreter, args: &[LoxValue]) -> LoxValue {
        match self {
            Function::Native(_, body) => body(args),
        }
    }

    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Function::Native(_, _) => write!(f, "<native fn>"),
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt(f)
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt(f)
    }
}
