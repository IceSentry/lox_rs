use crate::{
    environment::Environment,
    expr::Expr,
    function::Function,
    literal::Literal,
    logger::{Logger, LoggerImpl},
    lox::LoxValue,
    stmt::{Stmt, StmtResult},
    token::{Token, TokenType},
};
use std::{
    cell::RefCell,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

pub enum InterpreterError {
    RuntimeError(Token, String),
    Panic(Token, String),
}

pub struct Interpreter<'a> {
    environment: Rc<RefCell<Environment>>,
    // globals: Rc<RefCell<Environment>>,
    logger: &'a Rc<RefCell<LoggerImpl<'a>>>,
}

impl<'a> Interpreter<'a> {
    pub fn new(logger: &'a Rc<RefCell<LoggerImpl<'a>>>) -> Self {
        let globals = Rc::new(RefCell::new(Environment::default()));
        let clock_fn = Function::Native(
            0,
            Box::new(|_args: &Vec<LoxValue>| {
                LoxValue::Number(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Could not retrieve time.")
                        .as_millis() as f64,
                )
            }),
        );
        globals
            .borrow_mut()
            .declare(&String::from("clock"), LoxValue::Function(clock_fn));

        Interpreter {
            logger,
            environment: Rc::clone(&globals),
            // globals,
        }
    }

    pub fn interpret(&mut self, statements: &Vec<Stmt>) {
        for statement in statements {
            self.logger
                .borrow_mut()
                .println_debug(format!("{}", statement));
            if let Err(error) = self.execute(statement, self.environment.clone()) {
                self.logger.borrow_mut().runtime_error(error);
            }
        }
    }

    fn execute(
        &mut self,
        stmt: &Stmt,
        env: Rc<RefCell<Environment>>,
    ) -> Result<StmtResult, InterpreterError> {
        match stmt {
            Stmt::Expression(expr) => {
                let value = self.evaluate(&expr, &mut env.borrow_mut())?;
                self.logger.borrow_mut().println_repl(format!("{}", value));
                Ok(StmtResult::Unit)
            }
            Stmt::Print(expr) => {
                let value = self.evaluate(&expr, &mut env.borrow_mut())?;
                self.logger.borrow_mut().println(format!("{}", value));
                Ok(StmtResult::Unit)
            }
            Stmt::Let(token, initializer) => {
                let value = match initializer {
                    Some(inializer_value) => {
                        self.evaluate(&inializer_value, &mut env.borrow_mut())?
                    }
                    None => LoxValue::Nil,
                };
                env.borrow_mut().declare(&token.lexeme, value);
                Ok(StmtResult::Unit)
            }
            Stmt::Block(statements) => {
                let environment = Rc::new(RefCell::new(Environment::new(&env)));
                for stmt in statements {
                    match self.execute(stmt, environment.clone())? {
                        StmtResult::Break => return Ok(StmtResult::Break),
                        StmtResult::Continue => {
                            if env.borrow_mut().is_enclosing_loop() {
                                // This is to make sure the last block gets executed
                                // in a desugared for loop
                                return Ok(StmtResult::Continue);
                            }
                        }
                        StmtResult::Unit => (),
                    }
                }
                Ok(StmtResult::Unit)
            }
            Stmt::If(condition, then_branch, else_branch) => {
                if self
                    .evaluate(&condition, &mut env.borrow_mut())?
                    .is_truthy()
                {
                    self.execute(then_branch, env)
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch, env)
                } else {
                    Ok(StmtResult::Unit)
                }
            }
            Stmt::While(condition, body) => {
                while self
                    .evaluate(&condition, &mut env.borrow_mut())?
                    .is_truthy()
                {
                    env.borrow_mut().is_loop = true;
                    match self.execute(body, Rc::clone(&env))? {
                        StmtResult::Break => break,
                        StmtResult::Continue => continue,
                        StmtResult::Unit => (),
                    }
                }
                Ok(StmtResult::Unit)
            }
            Stmt::Break(token) => match env.borrow_mut().is_inside_loop() {
                true => Ok(StmtResult::Break),
                false => Err(self.error(&token, "'break' must be inside a loop")),
            },
            Stmt::Continue(token) => match env.borrow_mut().is_inside_loop() {
                true => Ok(StmtResult::Continue),
                false => Err(self.error(&token, "'continue' must be inside a loop")),
            },
        }
    }

    fn evaluate(
        &mut self,
        expr: &Expr,
        env: &mut Environment,
    ) -> Result<LoxValue, InterpreterError> {
        match expr {
            Expr::Binary(left, operator, right) => {
                self.evaluate_binary_op(left, operator, right, env)
            }
            Expr::Grouping(expr) => self.evaluate(expr, env),
            Expr::Literal(literal) => self.evaluate_literal(literal),
            Expr::Unary(operator, right) => self.evaluate_unary_op(operator, right, env),
            Expr::Variable(token) => match env.get(&token)? {
                LoxValue::Undefined => {
                    Err(self.error(token, &format!("{} is undefined!", token.lexeme)))
                }
                value => Ok(value),
            },
            Expr::Assign(token, value_expr) => {
                let value = self.evaluate(&value_expr, env)?;
                env.assign(token, value)
            }
            Expr::Logical(left, operator, right) => {
                let left = self.evaluate(left, env)?;
                match (&operator.token_type, left.is_truthy()) {
                    (TokenType::OR, true) => return Ok(left),
                    (TokenType::OR, false) => (),
                    (_, false) => return Ok(left),
                    (_, true) => (),
                }
                self.evaluate(right, env)
            }
            Expr::FunctionCall(callee, paren, args) => {
                let callee = self.evaluate(callee, env)?;
                let args: Result<Vec<LoxValue>, InterpreterError> =
                    args.iter().map(|arg| self.evaluate(arg, env)).collect();
                let args = args?;
                match callee {
                    LoxValue::Function(function) => {
                        if args.len() > function.arity() {
                            return Err(self.error(
                                paren,
                                &format!(
                                    "Expected {} arguments but got {}",
                                    function.arity(),
                                    args.len()
                                ),
                            ));
                        }
                        Ok(function.call(self, &args))
                    }
                    _ => Err(self.error(paren, "Can only call functions and classes")),
                }
            }
        }
    }

    fn evaluate_literal(&self, literal: &Literal) -> Result<LoxValue, InterpreterError> {
        Ok(match literal {
            Literal::String(value) => LoxValue::String(value.clone()),
            Literal::Number(value) => LoxValue::Number(*value),
            Literal::FALSE => LoxValue::Boolean(false),
            Literal::TRUE => LoxValue::Boolean(true),
            Literal::Nil => LoxValue::Nil,
        })
    }

    fn evaluate_unary_op(
        &mut self,
        operator: &Token,
        right: &Expr,
        env: &mut Environment,
    ) -> Result<LoxValue, InterpreterError> {
        let right = self.evaluate(&right, env)?;

        match operator.token_type {
            TokenType::BANG => Ok(LoxValue::Boolean(!right.is_truthy())),
            TokenType::MINUS => match right {
                LoxValue::Number(value) => Ok(LoxValue::Number(-value)),
                _ => Err(self.error(operator, "Operand must be a number")),
            },
            _ => unreachable!(),
        }
    }

    fn evaluate_binary_op(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
        env: &mut Environment,
    ) -> Result<LoxValue, InterpreterError> {
        let left = self.evaluate(&left, env)?;
        let right = self.evaluate(&right, env)?;

        use LoxValue::*;
        use TokenType::*;
        match (&operator.token_type, (&left, &right)) {
            (MINUS, (Number(left), Number(right))) => Ok(Number(left - right)),
            (SLASH, (Number(left), Number(right))) => match right == &0.0 {
                true => Err(self.error(operator, "Division by zero")),
                false => Ok(Number(left / right)),
            },
            (STAR, (Number(left), Number(right))) => Ok(Number(left * right)),
            (PLUS, (Number(left), Number(right))) => Ok(Number(left + right)),
            (PLUS, (String(left), String(right))) => Ok(String(format!("{}{}", left, right))),
            (PLUS, (String(left), Number(right))) => Ok(String(format!("{}{}", left, right))),
            (PLUS, _) => Err(self.error(operator, "Operands must be two numbers or two strings")),
            (GREATER, (Number(left), Number(right))) => Ok(Boolean(left > right)),
            (GREATER_EQUAL, (Number(left), Number(right))) => Ok(Boolean(left >= right)),
            (LESS, (Number(left), Number(right))) => Ok(Boolean(left < right)),
            (LESS_EQUAL, (Number(left), Number(right))) => Ok(Boolean(left <= right)),
            (BANG_EQUAL, _) => Ok(Boolean(!left.is_equal(right))),
            (EQUAL_EQUAL, _) => Ok(Boolean(left.is_equal(right))),
            (_, _) => self.error_number_operand(operator),
        }
    }

    fn error(&self, token: &Token, message: &str) -> InterpreterError {
        InterpreterError::RuntimeError(token.clone(), String::from(message))
    }

    fn error_number_operand(&self, token: &Token) -> Result<LoxValue, InterpreterError> {
        Err(InterpreterError::RuntimeError(
            token.clone(),
            String::from("Operands must be numbers"),
        ))
    }
}
