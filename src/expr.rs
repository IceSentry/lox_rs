use crate::scanner::{Literal, Token};
use std::fmt::*;

#[derive(Debug)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Literal),
    Unary(Token, Box<Expr>),
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Expr::Binary(left, operator, right) => write!(f, "({} {} {})", operator, left, right),
            Expr::Grouping(expression) => write!(f, "(group {})", expression),
            Expr::Literal(literal) => write!(f, "{}", literal),
            Expr::Unary(operator, right) => write!(f, "({} {})", operator, right),
        }
    }
}
