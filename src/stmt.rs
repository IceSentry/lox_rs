use crate::{expr::Expr, token::Token};
use std::fmt::*;

#[derive(Clone)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Block(Vec<Box<Stmt>>),
    Let(Token, Option<Expr>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
}

impl Display for Stmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Stmt::Expression(expression) => write!(f, "{}", expression),
            Stmt::Print(expression) => write!(f, "(print {})", expression),
            Stmt::Let(name, initializer) => match initializer {
                Some(value) => write!(f, "(let {} = {})", name, value),
                None => write!(f, "(let {} = None)", name),
            },
            Stmt::Block(statements) => {
                write!(f, "{{\n")?;
                for stmt in statements {
                    writeln!(f, "\t{}", stmt)?;
                }
                write!(f, "}}")
            }
            Stmt::If(condition, then_branch, else_branch) => match else_branch {
                Some(else_branch) => write!(
                    f,
                    "(if {} {{\n {} \n}} else {{\n {} \n}})",
                    condition, then_branch, else_branch
                ),
                None => write!(f, "(if {} {{\n {} \n}})", condition, then_branch),
            },
            Stmt::While(condition, body) => write!(f, "(while {} \n{}\n)", condition, body),
        }
    }
}
