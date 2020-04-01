use crate::{
    environment::Environment,
    interpreter::{Interpreter, RuntimeError},
    parser::Parser,
    scanner::Scanner,
    token::{Position, Token, TokenType},
};

use float_cmp::*;
use std::fmt::*;

#[derive(PartialEq, Clone)]
pub enum LoxValue {
    Nil,
    Number(f64),
    Boolean(bool),
    String(String),
}

impl LoxValue {
    pub fn is_truthy(self) -> bool {
        match self {
            LoxValue::Nil => false,
            LoxValue::Boolean(value) => value,
            _ => true,
        }
    }

    pub fn is_equal(self, other: LoxValue) -> bool {
        match (self, other) {
            (LoxValue::Nil, LoxValue::Nil) => true,
            (LoxValue::Number(a), LoxValue::Number(b)) => a.approx_eq(b, F64Margin::default()),
            (LoxValue::String(a), LoxValue::String(b)) => a == b,
            (LoxValue::Boolean(a), LoxValue::Boolean(b)) => a == b,
            _ => false, // no type coercion
        }
    }
}

impl Display for LoxValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            LoxValue::Nil => write!(f, "nil"),
            LoxValue::Number(value) => write!(f, "{}", value),
            LoxValue::Boolean(value) => write!(f, "{}", value),
            LoxValue::String(value) => write!(f, "{}", value),
        }
    }
}

pub struct Lox {
    pub had_error: bool,
    pub had_runtime_error: bool,
    pub debug: bool,
    pub is_repl: bool,
    pub environment: Environment,
}

impl Lox {
    pub fn new(debug: bool) -> Self {
        Lox {
            had_error: false,
            had_runtime_error: false,
            is_repl: false,
            debug,
            environment: Environment::default(),
        }
    }

    pub fn run<W>(&mut self, source: &str, output: W)
    where
        W: std::io::Write,
    {
        let mut scanner = Scanner::new(self, String::from(source));
        let tokens = scanner.scan_tokens();
        let mut parser = Parser::new(tokens.to_vec(), self);
        if let Ok(statements) = parser.parse() {
            let mut interpreter = Interpreter::new(self, output);
            interpreter.interpret(statements);
        }
    }

    pub fn error(&mut self, token: &Token, message: String) {
        match token.token_type {
            TokenType::EOF => self.report_error(token.position, " at end", message),
            _ => self.report_error(token.position, &format!(" at '{}'", token.lexeme), message),
        }
    }

    pub fn runtime_error(&mut self, error: RuntimeError) {
        let RuntimeError(token, message) = error;
        eprintln!("RuntimeError: {}\n[{}]", message, token.position);
        self.had_runtime_error = true;
    }

    pub fn report_error(&mut self, position: Position, error_where: &str, message: String) {
        eprintln!("[{}] Error{}: {}", position, error_where, message);
        self.had_error = true;
    }
}
