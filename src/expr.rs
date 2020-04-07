use crate::{literal::Literal, token::Token};
use std::fmt::{Debug, Display, Formatter, Result};

#[derive(Debug, Clone)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Literal),
    Unary(Token, Box<Expr>),
    Variable(Token),
    Assign(Token, Box<Expr>),
    Logical(Box<Expr>, Token, Box<Expr>),
    Call(Box<Expr>, Token, Vec<Expr>),
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Expr::Binary(left, operator, right) | Expr::Logical(left, operator, right) => {
                write!(f, "({} {} {})", left, operator, right)
            }
            Expr::Grouping(expression) => write!(f, "(group {})", expression),
            Expr::Literal(literal) => write!(f, "{}", literal),
            Expr::Unary(operator, right) => write!(f, "({} {})", operator, right),
            Expr::Variable(token) => write!(f, "{}", token),
            Expr::Assign(token, value) => write!(f, "({} = {})", token, value),
            Expr::Call(callee, _paren, args) => write!(f, "{}({:?})", callee, args),
        }
    }
}
