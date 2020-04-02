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
            TokenType::EOF => self.report_error(token.position, " at end", message),
            _ => self.report_error(token.position, &format!(" at '{}'", token.lexeme), message),
        }
    }

    fn runtime_error(&mut self, error: RuntimeError) {
        let RuntimeError(token, message) = error;
        self.println(format!("RuntimeError: {}\n[{}]", message, token.position));
    }

    fn report_error(&mut self, position: Position, error_where: &str, message: String) {
        self.println(format!("[{}] Error{}: {}", position, error_where, message));
    }
}

pub struct DefaultLogger {
    pub debug: bool,
    pub is_repl: bool,
    output: Stdout,
}

impl DefaultLogger {
    pub fn new() -> Self {
        DefaultLogger {
            debug: false,
            is_repl: false,
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

#[cfg(test)]
pub mod test {
    use crate::logger::Logger;
    use std::io::Write;

    pub struct TestLogger<'a> {
        pub output: &'a mut Vec<u8>,
    }

    impl<'a> TestLogger<'a> {
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
}
