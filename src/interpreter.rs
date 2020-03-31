use crate::{
    expr::Expr,
    scanner::{Literal, Token, TokenType},
    stmt::Stmt,
    Lox,
};
use derive_new::*;
use float_cmp::*;
use std::fmt;

pub struct RuntimeError(pub Token, pub String);

#[derive(PartialEq)]
pub enum LoxValue {
    Nil,
    Number(f64),
    Boolean(bool),
    String(String),
}

impl LoxValue {
    fn is_truthy(self) -> bool {
        match self {
            LoxValue::Nil => false,
            LoxValue::Boolean(value) => value,
            _ => true,
        }
    }

    fn is_equal(self, other: LoxValue) -> bool {
        match (self, other) {
            (LoxValue::Nil, LoxValue::Nil) => true,
            (LoxValue::Number(a), LoxValue::Number(b)) => a.approx_eq(b, F64Margin::default()),
            (LoxValue::String(a), LoxValue::String(b)) => a == b,
            (LoxValue::Boolean(a), LoxValue::Boolean(b)) => a == b,
            _ => false, // no type coercion
        }
    }
}

impl fmt::Display for LoxValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoxValue::Nil => write!(f, "nil"),
            LoxValue::Number(value) => write!(f, "{}", value),
            LoxValue::Boolean(value) => write!(f, "{}", value),
            LoxValue::String(value) => write!(f, "{}", value),
        }
    }
}

#[derive(new)]
pub struct Interpreter<'a> {
    lox: &'a mut Lox,
}

impl<'a> Interpreter<'a> {
    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        for statement in statements {
            if let Err(error) = self.execute(statement) {
                self.lox.runtime_error(error);
            }
        }
    }

    fn evaluate(&self, expr: Expr) -> Result<LoxValue, RuntimeError> {
        match expr {
            Expr::Binary(left, operator, right) => self.evaluate_binary_op(*left, operator, *right),
            Expr::Grouping(expr) => self.evaluate(*expr),
            Expr::Literal(literal) => self.evaluate_literal(literal),
            Expr::Unary(operator, right) => self.evaluate_unary_op(operator, *right),
        }
    }

    fn execute(&self, stmt: Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Expression(expr) => {
                self.evaluate(*expr)?;
                Ok(())
            }
            Stmt::Print(expr) => {
                let value = self.evaluate(*expr)?;
                println!("{}", value);
                Ok(())
            }
        }
    }

    fn evaluate_literal(&self, literal: Literal) -> Result<LoxValue, RuntimeError> {
        Ok(match literal {
            Literal::String(value) => LoxValue::String(value),
            Literal::Number(value) => LoxValue::Number(value),
            Literal::FALSE => LoxValue::Boolean(false),
            Literal::TRUE => LoxValue::Boolean(true),
            Literal::Nil => LoxValue::Nil,
            Literal::Identifier(_) => todo!(),
        })
    }

    fn evaluate_unary_op(&self, operator: Token, right: Expr) -> Result<LoxValue, RuntimeError> {
        let right = self.evaluate(right)?;

        match operator.token_type {
            TokenType::BANG => Ok(LoxValue::Boolean(!right.is_truthy())),
            TokenType::MINUS => match right {
                LoxValue::Number(value) => Ok(LoxValue::Number(-value)),
                _ => self.error(operator, "Operand must be a number"),
            },
            _ => unreachable!(),
        }
    }

    fn evaluate_binary_op(
        &self,
        left: Expr,
        operator: Token,
        right: Expr,
    ) -> Result<LoxValue, RuntimeError> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        use LoxValue::*;
        match operator.token_type {
            TokenType::MINUS => match (left, right) {
                (Number(left), Number(right)) => Ok(Number(left - right)),
                _ => self.error_number_operand(operator),
            },
            TokenType::SLASH => match (left, right) {
                (Number(left), Number(right)) => {
                    if right == 0.0 {
                        self.error(operator, "Division by zero")
                    } else {
                        Ok(Number(left / right))
                    }
                }
                _ => self.error_number_operand(operator),
            },
            TokenType::STAR => match (left, right) {
                (Number(left), Number(right)) => Ok(Number(left * right)),
                _ => self.error_number_operand(operator),
            },
            TokenType::PLUS => match (left, right) {
                (Number(left), Number(right)) => Ok(Number(left + right)),
                (String(left), String(right)) => Ok(String(format!("{}{}", left, right))),
                _ => self.error(operator, "Operands must be two numbers or two strings"),
            },
            TokenType::GREATER => match (left, right) {
                (Number(left), Number(right)) => Ok(Boolean(left > right)),
                _ => self.error_number_operand(operator),
            },
            TokenType::GREATER_EQUAL => match (left, right) {
                (Number(left), Number(right)) => Ok(Boolean(left >= right)),
                _ => self.error_number_operand(operator),
            },
            TokenType::LESS => match (left, right) {
                (Number(left), Number(right)) => Ok(Boolean(left < right)),
                _ => self.error_number_operand(operator),
            },
            TokenType::LESS_EQUAL => match (left, right) {
                (Number(left), Number(right)) => Ok(Boolean(left <= right)),
                _ => self.error_number_operand(operator),
            },
            TokenType::BANG_EQUAL => Ok(Boolean(!left.is_equal(right))),
            TokenType::EQUAL_EQUAL => Ok(Boolean(left.is_equal(right))),
            _ => unreachable!(),
        }
    }

    fn error(&self, token: Token, message: &str) -> Result<LoxValue, RuntimeError> {
        Err(RuntimeError(token, String::from(message)))
    }

    fn error_number_operand(&self, token: Token) -> Result<LoxValue, RuntimeError> {
        Err(RuntimeError(
            token,
            String::from("Operands must be numbers"),
        ))
    }
}
