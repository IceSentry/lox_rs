use crate::{expr::Expr, lox::LoxValue, token::Token};
use std::fmt::{Debug, Display, Formatter, Result};

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Block(Vec<Stmt>),
    Let(Token, Option<Expr>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    Break(Token),
    Continue(Token),
}

pub enum StmtResult {
    Break,
    Continue,
    Value(LoxValue),
}

impl From<LoxValue> for StmtResult {
    fn from(value: LoxValue) -> Self {
        StmtResult::Value(value)
    }
}

macro_rules! indent {
    ($dst:expr, $depth:expr) => {{
        writeln!($dst, "")?;
        for _ in 0..$depth {
            write!($dst, "    ")?; // use \t ?
        }
        Ok(())
    }};
    ($dst:expr, $depth:expr, $($arg:tt)*) => {{
        indent!($dst, $depth)?;
        write!($dst, $($arg)*)
    }};
}

fn write_block(f: &mut Formatter<'_>, stmts: &[Stmt], depth: i32) -> Result {
    match stmts.len() {
        0 => indent!(f, depth + 1, "(empty_block)"),
        1 => write_body(f, &stmts[0], depth + 1),
        _ => {
            write!(f, "{{")?;
            // indent!(f, depth, "{{")?;
            for stmt in stmts {
                write_body(f, stmt, depth + 1)?;
            }
            indent!(f, depth, "}}")
        }
    }
}

fn write_body(f: &mut Formatter<'_>, body: &Stmt, depth: i32) -> Result {
    match body {
        Stmt::Block(stmts) => write_block(f, stmts, depth),
        Stmt::If(condition, then_branch, else_branch) => {
            write_if(f, condition, then_branch, else_branch, depth)
        }
        Stmt::While(condition, body) => write_while(f, condition, body, depth),
        _ => indent!(f, depth, "{}", body),
    }
}

fn write_if(
    f: &mut Formatter<'_>,
    condition: &Expr,
    then_branch: &Stmt,
    else_branch: &Option<Box<Stmt>>,
    depth: i32,
) -> Result {
    indent!(f, depth, "(if {} ", condition)?;
    write_body(f, then_branch, depth)?;
    if let Some(else_branch) = else_branch {
        indent!(f, depth, "else ")?;
        write_body(f, else_branch, depth)?;
    }
    write!(f, ")")
}

fn write_while(f: &mut Formatter<'_>, condition: &Expr, body: &Stmt, depth: i32) -> Result {
    indent!(f, depth, "(while {} ", condition)?;
    write_body(f, body, depth)?;
    write!(f, ")")
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
            Stmt::Block(statements) => write_block(f, statements, 0),
            Stmt::If(condition, then_branch, else_branch) => {
                write_if(f, condition, then_branch, else_branch, 0)
            }
            Stmt::While(condition, body) => write_while(f, condition, body, 0),
            Stmt::Break(_token) => write!(f, "(break)"),
            Stmt::Continue(_token) => write!(f, "(continue)"),
        }
    }
}
