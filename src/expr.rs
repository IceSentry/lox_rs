use crate::{literal::Literal, token::Token};
use std::fmt::*;

#[derive(Debug)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Literal),
    Unary(Token, Box<Expr>),
    Variable(Token),
    Assign(Token, Box<Expr>),
    Logical(Box<Expr>, Token, Box<Expr>),
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Expr::Binary(left, operator, right) => write!(f, "({} {} {})", left, operator, right),
            Expr::Grouping(expression) => write!(f, "(group {})", expression),
            Expr::Literal(literal) => write!(f, "{}", literal),
            Expr::Unary(operator, right) => write!(f, "({} {})", operator, right),
            Expr::Variable(token) => write!(f, "{}", token),
            Expr::Assign(token, value) => write!(f, "{} = {}", token, value),
            Expr::Logical(left, operator, right) => write!(f, "({} {} {})", left, operator, right),
        }
    }
}
