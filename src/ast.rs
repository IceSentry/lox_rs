use crate::{
    lox::LoxValue,
    token::{Literal, Token},
};
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

impl Stmt {
    fn fmt(&self, f: &mut Formatter<'_>, depth: i32) -> Result {
        match self {
            Stmt::Expression(expression) => write!(f, "{}", expression),
            Stmt::Print(expression) => write!(f, "(print {})", expression),
            Stmt::Let(name, initializer) => match initializer {
                Some(value) => write!(f, "(let {} = {})", name, value),
                None => write!(f, "(let {} = None)", name),
            },
            Stmt::Block(statements) => match statements.len() {
                0 => indent!(f, depth + 1, "(empty_block)"),
                1 => write_body(f, &statements[0], depth + 1),
                _ => {
                    write!(f, "{{")?;
                    for stmt in statements {
                        write_body(f, stmt, depth + 1)?;
                    }
                    indent!(f, depth, "}}")
                }
            },
            Stmt::If(condition, then_branch, else_branch) => {
                indent!(f, depth, "(if {} ", condition)?;
                write_body(f, then_branch, depth)?;
                if let Some(else_branch) = else_branch {
                    indent!(f, depth, "else ")?;
                    write_body(f, else_branch, depth)?;
                }
                write!(f, ")")
            }
            Stmt::While(condition, body) => {
                indent!(f, depth, "(while {} ", condition)?;
                write_body(f, body, depth)?;
                write!(f, ")")
            }
            Stmt::Break(_token) => write!(f, "(break)"),
            Stmt::Continue(_token) => write!(f, "(continue)"),
        }
    }
}

fn write_body(f: &mut Formatter<'_>, body: &Stmt, depth: i32) -> Result {
    match body {
        Stmt::Block(_) | Stmt::If(_, _, _) | Stmt::While(_, _) => body.fmt(f, depth),
        _ => indent!(f, depth, "{}", body),
    }
}

impl Display for Stmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.fmt(f, 0)
    }
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
