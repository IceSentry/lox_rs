use derive_new::new;
use std::fmt::{Debug, Display, Formatter, Result};

#[rustfmt::skip]
#[allow(non_camel_case_types, clippy::module_name_repetitions)]
#[derive(Debug, PartialEq, Clone, Copy)]
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
    PRINT, RETURN, SUPER, THIS, TRUE, LET, WHILE,
    LOOP, BREAK, CONTINUE,

    EOF
}

#[derive(new, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub position: Position,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.lexeme)
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.token_type {
            TokenType::IDENTIFIER => write!(f, "{:?} {}", &self.token_type, &self.lexeme),
            _ => write!(f, "{:?}", &self.token_type),
        }
    }
}

#[derive(new, Clone, Copy, Debug)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn increment_line(&mut self) {
        self.line += 1;
        self.column = 1;
    }

    pub fn increment_column(&mut self) {
        self.column += 1;
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "ln {} col {}", self.line, self.column)
    }
}

#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Number(f64),
    FALSE,
    TRUE,
    Nil, // TODO remove Nil when Option<LoxValue> exists
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Literal::Number(value) => write!(f, "{}", value),
            Literal::String(value) => write!(f, "\"{}\"", value),
            Literal::FALSE => write!(f, "false"),
            Literal::TRUE => write!(f, "true"),
            Literal::Nil => write!(f, "nil"),
        }
    }
}
