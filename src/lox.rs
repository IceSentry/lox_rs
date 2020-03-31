use crate::{
    interpreter::{Interpreter, RuntimeError},
    parser::Parser,
    scanner::{Position, Scanner, Token, TokenType},
    stmt::Stmt,
};

pub struct Lox {
    pub had_error: bool,
    pub had_runtime_error: bool,
}

impl Lox {
    pub fn new() -> Self {
        Lox {
            had_error: false,
            had_runtime_error: false,
        }
    }

    pub fn run(&mut self, source: &str) {
        let mut scanner = Scanner::new(self, String::from(source));
        let tokens = scanner.scan_tokens();
        let mut parser = Parser::new(tokens.to_vec(), self);
        if let Ok(statements) = parser.parse() {
            let mut interpreter = Interpreter::new(self);
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
