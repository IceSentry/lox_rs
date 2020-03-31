use crate::Lox;
use derive_new::*;
use fmt::Display;
use std::fmt;
use std::fmt::*;

#[rustfmt::skip]
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    // Single-character tokens
    LEFT_PAREN, RIGHT_PAREN, LEFT_BRACE, RIGHT_BRACE, 
    COMMA, DOT, MINUS, PLUS, SLASH, STAR, SEMICOLON,

    // One or two char tokens
    BANG, BANG_EQUAL,
    EQUAL, EQUAL_EQUAL,
    GREATER, GREATER_EQUAL,                          
    LESS, LESS_EQUAL,

    // Literals
    IDENTIFIER, STRING, NUMBER,

    //Keywords
    AND, CLASS, ELSE, FALSE, FUN, FOR, IF, NIL, OR,
    PRINT, RETURN, SUPER, THIS, TRUE, VAR, WHILE,

    EOF
}

#[derive(Debug, Clone)]
pub enum Literal {
    Identifier(String),
    String(String),
    Number(f64),
    FALSE,
    TRUE,
    Nil,
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Literal::Number(value) => write!(f, "{}", value),
            Literal::String(value) => write!(f, "{}", value),
            Literal::Identifier(identifier) => write!(f, "{}", identifier),
            Literal::FALSE => write!(f, "false"),
            Literal::TRUE => write!(f, "true"),
            Literal::Nil => write!(f, "nil"),
        }
    }
}

#[derive(new, Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub position: Position,
}

impl Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.lexeme)
    }
}

#[derive(new, Clone, Copy, Debug)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    fn increment_line(&mut self) {
        self.line += 1;
        self.column = 1;
    }

    fn increment_column(&mut self) {
        self.column += 1;
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ln {} col {}", self.line, self.column)
    }
}

pub struct Scanner<'a> {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    position: Position,
    lox: &'a mut Lox,
}

fn is_alphanumeric(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl<'a> Scanner<'a> {
    pub fn new(lox: &'a mut Lox, source: String) -> Self {
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            position: Position { line: 1, column: 1 },
            lox,
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
        use TokenType::*;
        let c = self.advance();
        match c {
            '(' => Some((LEFT_PAREN, None)),
            ')' => Some((RIGHT_PAREN, None)),
            '{' => Some((LEFT_BRACE, None)),
            '}' => Some((RIGHT_BRACE, None)),
            ',' => Some((COMMA, None)),
            '.' => Some((DOT, None)),
            '-' => Some((MINUS, None)),
            '+' => Some((PLUS, None)),
            ';' => Some((SEMICOLON, None)),
            '*' => Some((STAR, None)),
            '!' => Some((
                if self.advance_if_match('=') {
                    BANG_EQUAL
                } else {
                    BANG
                },
                None,
            )),
            '=' => Some((
                if self.advance_if_match('=') {
                    EQUAL_EQUAL
                } else {
                    EQUAL
                },
                None,
            )),
            '<' => Some((
                if self.advance_if_match('=') {
                    LESS_EQUAL
                } else {
                    LESS
                },
                None,
            )),
            '>' => Some((
                if self.advance_if_match('=') {
                    GREATER_EQUAL
                } else {
                    GREATER
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
                self.lox.report_error(
                    self.position,
                    "Scanner",
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
        match self.source.chars().nth(self.current) {
            Some(c) => {
                if c != expected {
                    false
                } else {
                    self.current += 1;
                    true
                }
            }
            None => false,
        }
    }

    fn string(&mut self) -> Option<(TokenType, Option<Literal>)> {
        while self.peek() != '"' && !self.is_at_end() {
            self.advance();
        }

        if self.is_at_end() {
            self.lox.report_error(
                self.position,
                "Scanner",
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
                    self.lox.report_error(
                        self.position,
                        "Scanner",
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
            "var" => (VAR, None),
            "while" => (WHILE, None),
            _ => (
                IDENTIFIER,
                Some(Literal::Identifier(String::from(current_lexeme))),
            ),
        }
    }
}
