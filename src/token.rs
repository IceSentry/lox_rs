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
    PRINT, RETURN, SUPER, THIS, TRUE, LET, WHILE,

    EOF
}

use crate::literal::Literal;
use derive_new::*;
use fmt::Display;
use std::fmt;
use std::fmt::*;

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
    pub fn increment_line(&mut self) {
        self.line += 1;
        self.column = 1;
    }

    pub fn increment_column(&mut self) {
        self.column += 1;
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ln {} col {}", self.line, self.column)
    }
}
