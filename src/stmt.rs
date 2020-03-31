use crate::expr::Expr;
use std::fmt::*;

pub enum Stmt {
    Expression(Box<Expr>),
    Print(Box<Expr>),
}

impl Display for Stmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Stmt::Expression(expression) => write!(f, "{}", expression),
            Stmt::Print(expression) => write!(f, "(print {})", expression),
        }
    }
}
