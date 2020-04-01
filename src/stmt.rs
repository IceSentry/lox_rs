use crate::{expr::Expr, token::Token};
use std::fmt::*;

pub enum Stmt {
    Expression(Box<Expr>),
    Print(Box<Expr>),
    Let(Token, Box<Option<Expr>>),
    Block(Vec<Box<Stmt>>),
}

impl Display for Stmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Stmt::Expression(expression) => write!(f, "{}", expression),
            Stmt::Print(expression) => write!(f, "(print {})", expression),
            Stmt::Let(name, initializer) => write!(f, "(let {} = {:?})", name, initializer),
            Stmt::Block(statements) => {
                write!(f, "{{")?;
                for stmt in statements {
                    writeln!(f, "\t{}", stmt)?;
                }
                write!(f, "}}")
            }
        }
    }
}
