use crate::{
    environment::Environment,
    expr::Expr,
    literal::Literal,
    lox::LoxValue,
    stmt::Stmt,
    token::{Token, TokenType},
    Lox,
};
use std::{cell::RefCell, rc::Rc};

pub struct RuntimeError(pub Token, pub String);

pub struct Interpreter<'a, W>
where
    W: std::io::Write,
{
    lox: &'a mut Lox,
    output: W,
}

impl<'a, W> Interpreter<'a, W>
where
    W: std::io::Write,
{
    pub fn new(lox: &'a mut Lox, output: W) -> Self {
        Interpreter { lox, output }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        for statement in statements {
            if self.lox.debug {
                println!("DEBUG {}", statement);
            }
            if let Err(error) = self.execute(statement) {
                self.lox.runtime_error(error);
            }
        }
    }

    fn evaluate(&mut self, expr: Expr) -> Result<LoxValue, RuntimeError> {
        match expr {
            Expr::Binary(left, operator, right) => self.evaluate_binary_op(*left, operator, *right),
            Expr::Grouping(expr) => self.evaluate(*expr),
            Expr::Literal(literal) => self.evaluate_literal(literal),
            Expr::Unary(operator, right) => self.evaluate_unary_op(operator, *right),
            Expr::Variable(token) => match self.lox.environment.get(&token)? {
                LoxValue::Undefined => {
                    self.error(token.clone(), &format!("{} is undefined!", token.lexeme))
                }
                value => Ok(value),
            },
            Expr::Assign(token, value_expr) => {
                let value = self.evaluate(*value_expr)?;
                self.lox.environment.assign(token, value.clone())
            }
        }
    }

    fn execute(&mut self, stmt: Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Expression(expr) => {
                let value = self.evaluate(*expr)?;
                if self.lox.debug || self.lox.is_repl {
                    println!("=> {}", value);
                }
                Ok(())
            }
            Stmt::Print(expr) => {
                let value = self.evaluate(*expr)?;
                writeln!(self.output, "{}", value).expect("failed to write to ouput");
                Ok(())
            }
            Stmt::Let(token, initializer) => {
                let value = match *initializer {
                    Some(inializer_value) => self.evaluate(inializer_value)?,
                    None => LoxValue::Nil,
                };
                self.lox.environment.declare(token.lexeme, value);
                Ok(())
            }
            Stmt::Block(statements) => self.execute_block(
                statements,
                Environment::new(Rc::new(RefCell::new(self.lox.environment.clone()))),
            ),
        }
    }

    fn execute_block(
        &mut self,
        statements: Vec<Box<Stmt>>,
        environment: Environment,
    ) -> Result<(), RuntimeError> {
        let previous = self.lox.environment.clone();
        self.lox.environment = environment;

        for stmt in statements {
            self.execute(*stmt)?;
        }

        self.lox.environment = previous;

        Ok(())
    }

    fn evaluate_literal(&self, literal: Literal) -> Result<LoxValue, RuntimeError> {
        Ok(match literal {
            Literal::String(value) => LoxValue::String(value),
            Literal::Number(value) => LoxValue::Number(value),
            Literal::FALSE => LoxValue::Boolean(false),
            Literal::TRUE => LoxValue::Boolean(true),
            Literal::Nil => LoxValue::Nil,
        })
    }

    fn evaluate_unary_op(
        &mut self,
        operator: Token,
        right: Expr,
    ) -> Result<LoxValue, RuntimeError> {
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
        &mut self,
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

#[cfg(test)]
mod tests {
    use crate::lox::Lox;

    #[test]
    fn test_shadowing() {
        let source = r#"
            let a = 1;
            {
                let a = a + 2;
                print a; // 3
            }
        "#;
        let mut lox = Lox::new(false);
        let mut output = Vec::new();
        lox.run(source, &mut output);
        let output = String::from_utf8(output).expect("Not UTF-8");
        assert_eq!("3", output.trim());
    }

    #[test]
    fn test_block() {
        let source = r#"
            let a = "global a";
            let b = "global b";
            let c = "global c";
            {
                let a = "outer a";
                let b = "outer b";
                {
                    let a = "inner a";
                    print a;
                    print b;
                    print c;
                }
                print a;
                print b;
                print c;
            }
            print a;
            print b;
            print c;
        "#;
        let mut lox = Lox::new(false);
        let mut output = Vec::new();
        lox.run(source, &mut output);
        let output = String::from_utf8(output).expect("Not UTF-8");
        let mut output = output.split('\n').into_iter();

        assert_eq!("inner a", output.next().unwrap());
        assert_eq!("outer b", output.next().unwrap());
        assert_eq!("global c", output.next().unwrap());
        assert_eq!("outer a", output.next().unwrap());
        assert_eq!("outer b", output.next().unwrap());
        assert_eq!("global c", output.next().unwrap());
        assert_eq!("global a", output.next().unwrap());
        assert_eq!("global b", output.next().unwrap());
        assert_eq!("global c", output.next().unwrap());
    }

    #[test]
    fn test_operator_precedence() {
        let source = r#"
            print 2 + 3 * 4 * 5 - 6;
        "#;
        let mut lox = Lox::new(false);
        let mut output = Vec::new();
        lox.run(source, &mut output);
        let output = String::from_utf8(output).expect("Not UTF-8");
        assert_eq!("56", output.trim());
    }
}
