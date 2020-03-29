use crate::Lox;
use derive_new::*;
use std::fmt;

#[rustfmt::skip]
#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum TokenType {
    // Single-character tokens
    LEFT_PAREN, RIGHT_PAREN, LEFT_BRACE, RIGHT_BRACE, COMMA, DOT, MINUS, PLUS, SLASH, STAR, SEMICOLON,

    // One or two char tokens
    BANG, BANG_EQUAL,
    EQUAL, EQUAL_EQUAL,
    GREATER, GREATER_EQUAL,                          
    LESS, LESS_EQUAL,

    // Literals
    IDENTIFIER(String), STRING(String), NUMBER(f64),

    //Keywords
    AND, CLASS, ELSE, FALSE, FUN, FOR, IF, NIL, OR,
    PRINT, RETURN, SUPER, THIS, TRUE, VAR, WHILE,

    EOF
}

#[derive(new, Debug)]
pub struct Token {
    token_type: TokenType,
    lexeme: String,
    position: Position,
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
            if let Some(token) = self.scan_token() {
                self.add_token(token);
            }
        }
        self.tokens
            .push(Token::new(TokenType::EOF, String::from(""), self.position));
        &self.tokens
    }

    fn scan_token(&mut self) -> Option<TokenType> {
        use TokenType::*;
        let c = self.advance();
        match c {
            '(' => Some(LEFT_PAREN),
            ')' => Some(RIGHT_PAREN),
            '{' => Some(LEFT_BRACE),
            '}' => Some(RIGHT_BRACE),
            ',' => Some(COMMA),
            '.' => Some(DOT),
            '-' => Some(MINUS),
            '+' => Some(PLUS),
            ';' => Some(SEMICOLON),
            '*' => Some(STAR),
            '!' => Some(if self.advance_if_match('=') {
                BANG_EQUAL
            } else {
                BANG
            }),
            '=' => Some(if self.advance_if_match('=') {
                EQUAL_EQUAL
            } else {
                EQUAL
            }),
            '<' => Some(if self.advance_if_match('=') {
                LESS_EQUAL
            } else {
                LESS
            }),
            '>' => Some(if self.advance_if_match('=') {
                GREATER_EQUAL
            } else {
                GREATER
            }),
            '/' => {
                if self.advance_if_match('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                    None
                } else if self.advance_if_match('*') {
                    println!("adv if match *");

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
                    Some(TokenType::SLASH)
                }
            }
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

    fn add_token(&mut self, token: TokenType) {
        let text = &self.source[self.start..self.current];
        self.tokens
            .push(Token::new(token, String::from(text), self.position));
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

    fn string(&mut self) -> Option<TokenType> {
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

        Some(TokenType::STRING(String::from(
            &self.source[(self.start + 1)..(self.current - 1)],
        )))
    }

    fn number(&mut self) -> TokenType {
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
        TokenType::NUMBER(value)
    }

    fn identifier(&mut self) -> TokenType {
        use TokenType::*;
        while is_alphanumeric(self.peek()) {
            self.advance();
        }
        let current_lexeme = &self.source[self.start..self.current];
        match current_lexeme {
            "and" => AND,
            "class" => CLASS,
            "else" => ELSE,
            "false" => FALSE,
            "for" => FOR,
            "fun" => FUN,
            "if" => IF,
            "nil" => NIL,
            "or" => OR,
            "print" => PRINT,
            "return" => RETURN,
            "super" => SUPER,
            "this" => THIS,
            "true" => TRUE,
            "var" => VAR,
            "while" => WHILE,
            _ => IDENTIFIER(String::from(current_lexeme)),
        }
    }
}
