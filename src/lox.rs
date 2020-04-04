use crate::{
    function::Function, interpreter::Interpreter, logger::LoggerImpl, parser::Parser,
    scanner::Scanner, token::Token,
};

use derive_new::*;
use float_cmp::*;
use std::{cell::RefCell, fmt, rc::Rc};

#[derive(Clone)]
pub enum LoxValue {
    Nil,       // TODO implement Option<T> and remove nil
    Undefined, // This is used as a flag, there are no corresponding literal
    Number(f64),
    Boolean(bool),
    String(String),
    Function(Function),
    Unit,
}

impl LoxValue {
    pub fn is_truthy(&self) -> bool {
        use LoxValue::*;
        match self {
            Nil | Undefined => false,
            Boolean(value) => *value,
            _ => true,
        }
    }

    pub fn is_equal(&self, other: LoxValue) -> bool {
        use LoxValue::*;
        match (self, other) {
            (Nil, Nil) => true,
            (Number(a), Number(b)) => a.approx_eq(b, F64Margin::default()),
            (String(ref a), String(ref b)) => *a == *b,
            (Boolean(a), Boolean(b)) => *a == b,
            _ => false, // no type coercion
        }
    }
}

impl fmt::Display for LoxValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LoxValue::*;
        match self {
            Nil => write!(f, "nil"),
            Undefined => write!(f, "undefined"),
            Number(value) => write!(f, "{}", value),
            Boolean(value) => write!(f, "{}", value),
            String(value) => write!(f, "{}", value),
            Function(function) => function.fmt(f),
            Unit => write!(f, "()"),
        }
    }
}

pub struct Lox<'a> {
    pub logger: &'a Rc<RefCell<LoggerImpl<'a>>>,
    pub interpreter: Interpreter<'a>,
}

#[derive(new)]
pub struct ErrorData {
    pub token: Token,
    pub message: String,
}

pub enum LoxError {
    Parser(ErrorData),
    Runtime(ErrorData),
    Panic(ErrorData),
}

pub type LoxResult<T> = std::result::Result<T, LoxError>;

impl<'a> Lox<'a> {
    pub fn new(logger: &'a Rc<RefCell<LoggerImpl<'a>>>) -> Self {
        Lox {
            logger,
            interpreter: Interpreter::new(logger),
        }
    }

    pub fn run(&mut self, source: &str) -> LoxResult<()> {
        let mut scanner = Scanner::new(self.logger, String::from(source));
        let tokens = scanner.scan_tokens();
        let mut parser = Parser::new(tokens.to_vec(), self.logger);
        let statements = parser.parse()?;
        self.interpreter.interpret(&statements);
        Ok(())
    }
}
