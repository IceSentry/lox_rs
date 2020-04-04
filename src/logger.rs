use crate::{
    lox::LoxError,
    token::{Position, Token, TokenType},
};
use enum_dispatch::*;
use std::io::Stdout;
use std::io::Write;

#[enum_dispatch(Logger)]
pub enum LoggerImpl<'a> {
    DefaultLogger(DefaultLogger),
    TestLogger(TestLogger<'a>),
}

#[enum_dispatch]
pub trait Logger {
    fn println(&mut self, message: String);
    fn println_debug(&mut self, message: String);
    fn println_repl(&mut self, message: String);

    fn error(&mut self, token: &Token, message: String) {
        match token.token_type {
            TokenType::EOF => self.report_error(token.position, "Parser", "at end", message),
            _ => self.report_error(
                token.position,
                "Parser",
                &format!("at '{}'", token.lexeme),
                message,
            ),
        }
    }

    fn runtime_error(&mut self, error: LoxError) {
        match error {
            LoxError::Runtime(err) => {
                self.report_error(err.token.position, "Runtime", "", err.message);
            }
            LoxError::Panic(err) => {
                self.report_error(err.token.position, "Panic!", "", err.message);
                std::process::exit(70)
            }
            LoxError::Parser(_) => unreachable!(),
        }
    }

    fn report_error(&mut self, position: Position, tag: &str, error_where: &str, message: String) {
        self.println(format!(
            "[{}] {}Error {}: {}",
            position, tag, error_where, message
        ));
    }
}

pub struct DefaultLogger {
    pub debug: bool,
    pub is_repl: bool,
    output: Stdout,
}

impl DefaultLogger {
    pub fn new(debug: bool, is_repl: bool) -> Self {
        DefaultLogger {
            debug,
            is_repl,
            output: std::io::stdout(),
        }
    }
}

impl Logger for DefaultLogger {
    fn println(&mut self, message: String) {
        writeln!(self.output, "{}", message).expect("Failed to write");
    }

    fn println_debug(&mut self, message: String) {
        if self.debug {
            self.println(format!("DEBUG {}", message));
        }
    }

    fn println_repl(&mut self, message: String) {
        if self.is_repl {
            self.println(format!("=> {}", message));
        }
    }
}

pub struct TestLogger<'a> {
    pub output: &'a mut Vec<u8>,
}

impl<'a> TestLogger<'a> {
    #[allow(dead_code)]
    pub fn new(output: &'a mut Vec<u8>) -> Self {
        TestLogger { output }
    }
}

impl<'a> Logger for TestLogger<'a> {
    fn println(&mut self, message: String) {
        writeln!(self.output, "{}", message).expect("Failed to write");
    }

    fn println_debug(&mut self, _message: String) {}
    fn println_repl(&mut self, _message: String) {}
}
