use crate::{
    interpreter::RuntimeError,
    token::{Position, Token, TokenType},
};
use std::io::Stdout;
use std::io::Write;

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

    fn runtime_error(&mut self, error: RuntimeError) {
        let RuntimeError(token, message) = error;
        self.report_error(token.position, "Runtime", "", message);
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
