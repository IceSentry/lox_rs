use crate::{
    environment::Environment,
    function::Function,
    interpreter::Interpreter,
    logger::Logger,
    parser::{Parser, ParserError},
    scanner::Scanner,
};

use float_cmp::*;
use std::{
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

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
    pub environment: Environment,
    pub logger: &'a mut dyn Logger,
}

pub enum LoxError {
    Parser(ParserError),
}

impl<'a> Lox<'a> {
    pub fn new(logger: &'a mut dyn Logger) -> Self {
        let globals = Environment::default();
        let clock_fn = Function::Native(
            0,
            Box::new(|_args: &Vec<LoxValue>| {
                LoxValue::Number(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Could not retrieve time.")
                        .as_millis() as f64,
                )
            }),
        );
        globals.declare(&String::from("clock"), LoxValue::Function(clock_fn));
        Lox {
            logger,
            environment: globals,
        }
    }

    pub fn run(&mut self, source: &str) -> Result<(), LoxError> {
        let mut scanner = Scanner::new(self.logger, String::from(source));
        let tokens = scanner.scan_tokens();
        let mut parser = Parser::new(tokens.to_vec(), self.logger);
        match parser.parse() {
            Ok(statements) => {
                let mut interpreter = Interpreter::new(self.logger);
                interpreter.interpret(&statements, &mut self.environment);
                Ok(())
            }
            Err(err) => Err(LoxError::Parser(err)),
        }
    }
}
