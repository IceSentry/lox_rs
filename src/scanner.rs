use crate::{
    logger::{Logger, LoggerImpl},
    token::{Literal, Position, Token, TokenType},
};
use std::{cell::RefCell, rc::Rc};

pub struct Scanner<'a> {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    position: Position,
    logger: &'a Rc<RefCell<LoggerImpl<'a>>>,
}

fn is_alphanumeric(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl<'a> Scanner<'a> {
    pub fn new(logger: &'a Rc<RefCell<LoggerImpl<'a>>>, source: String) -> Self {
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            position: Position { line: 1, column: 1 },
            logger,
        }
    }

    pub fn scan_tokens(&mut self) -> &[Token] {
        while !self.is_at_end() {
            self.start = self.current;
            if let Some((token, literal)) = self.scan_token() {
                self.add_token(token, literal);
            }
        }
        self.tokens.push(Token::new(
            TokenType::EOF,
            String::from(""),
            None,
            self.position,
        ));
        &self.tokens
    }

    fn scan_token(&mut self) -> Option<(TokenType, Option<Literal>)> {
        let c = self.advance();
        match c {
            '(' => Some((TokenType::LEFT_PAREN, None)),
            ')' => Some((TokenType::RIGHT_PAREN, None)),
            '{' => Some((TokenType::LEFT_BRACE, None)),
            '}' => Some((TokenType::RIGHT_BRACE, None)),
            ',' => Some((TokenType::COMMA, None)),
            '.' => Some((TokenType::DOT, None)),
            '-' => Some((TokenType::MINUS, None)),
            '+' => Some((TokenType::PLUS, None)),
            ';' => Some((TokenType::SEMICOLON, None)),
            '*' => Some((TokenType::STAR, None)),
            '!' => Some((
                if self.advance_if_match('=') {
                    TokenType::BANG_EQUAL
                } else {
                    TokenType::BANG
                },
                None,
            )),
            '=' => Some((
                if self.advance_if_match('=') {
                    TokenType::EQUAL_EQUAL
                } else {
                    TokenType::EQUAL
                },
                None,
            )),
            '<' => Some((
                if self.advance_if_match('=') {
                    TokenType::LESS_EQUAL
                } else {
                    TokenType::LESS
                },
                None,
            )),
            '>' => Some((
                if self.advance_if_match('=') {
                    TokenType::GREATER_EQUAL
                } else {
                    TokenType::GREATER
                },
                None,
            )),
            '/' => self.comment_or_slash(),
            ' ' | '\r' | '\t' => None, // ignore whitespace
            '\n' => {
                self.position.increment_line();
                None
            }
            '"' => self.string(),
            c if c.is_ascii_digit() => Some(self.number()),
            c if is_alphanumeric(c) => Some(self.identifier()),
            _ => {
                self.logger.borrow_mut().report_error(
                    self.position,
                    "Scanner",
                    "",
                    format!("Unexpected character \"{}\"", c),
                );
                None
            }
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn add_token(&mut self, token: TokenType, literal: Option<Literal>) {
        let text = &self.source[self.start..self.current];
        self.tokens.push(Token::new(
            token,
            String::from(text),
            literal,
            self.position,
        ));
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source
            .chars()
            .nth(self.current)
            .expect("current should exist")
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.source
            .chars()
            .nth(self.current + 1)
            .expect("current + 1 should exist")
    }

    fn advance(&mut self) -> char {
        let c = self
            .source
            .chars()
            .nth(self.current)
            .expect("current should exist");
        self.current += 1;
        if c == '\n' {
            self.position.increment_line()
        } else {
            self.position.increment_column()
        }
        c
    }

    fn advance_if_match(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if let Some(c) = self.source.chars().nth(self.current) {
            if c == expected {
                self.current += 1;
                return true;
            }
        }
        false
    }

    fn string(&mut self) -> Option<(TokenType, Option<Literal>)> {
        while self.peek() != '"' && !self.is_at_end() {
            self.advance();
        }

        if self.is_at_end() {
            self.logger.borrow_mut().report_error(
                self.position,
                "Scanner",
                "",
                String::from("Unterminated string"),
            );
            return None;
        }

        self.advance(); // the closing "

        Some((
            TokenType::STRING,
            Some(Literal::String(String::from(
                &self.source[(self.start + 1)..(self.current - 1)],
            ))),
        ))
    }

    fn number(&mut self) -> (TokenType, Option<Literal>) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance(); //consume the .
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        let current_lexeme = &self.source[self.start..self.current];
        let value: f64 = current_lexeme
            .parse()
            .expect("current_lexeme should be a number");
        (TokenType::NUMBER, Some(Literal::Number(value)))
    }

    fn comment_or_slash(&mut self) -> Option<(TokenType, Option<Literal>)> {
        if self.advance_if_match('/') {
            while self.peek() != '\n' && !self.is_at_end() {
                self.advance();
            }
            None
        } else if self.advance_if_match('*') {
            loop {
                if self.peek() == '*' && self.peek_next() == '/' {
                    self.advance();
                    self.advance(); // consume */
                    return None;
                }
                if self.is_at_end() {
                    self.logger.borrow_mut().report_error(
                        self.position,
                        "Scanner",
                        "",
                        String::from("Unterminated block comment"),
                    );
                }
                self.advance();
            }
        } else {
            Some((TokenType::SLASH, None))
        }
    }

    fn identifier(&mut self) -> (TokenType, Option<Literal>) {
        use TokenType::*;
        while is_alphanumeric(self.peek()) {
            self.advance();
        }
        let current_lexeme = &self.source[self.start..self.current];
        match current_lexeme {
            "and" => (AND, None),
            "class" => (CLASS, None),
            "else" => (ELSE, None),
            "false" => (FALSE, Some(Literal::FALSE)),
            "for" => (FOR, None),
            "fun" => (FUN, None),
            "if" => (IF, None),
            "nil" => (NIL, Some(Literal::Nil)),
            "or" => (OR, None),
            "print" => (PRINT, None),
            "return" => (RETURN, None),
            "super" => (SUPER, None),
            "this" => (THIS, None),
            "true" => (TRUE, Some(Literal::TRUE)),
            "let" | "var" => (LET, None),
            "while" => (WHILE, None),
            "loop" => (LOOP, None),
            "break" => (BREAK, None),
            "continue" => (CONTINUE, None),
            _ => (IDENTIFIER, None),
        }
    }
}
