use crate::{
    environment::Environment,
    expr::Expr,
    function::Function,
    logger::{Logger, LoggerImpl},
    lox::{ErrorData, LoxError, LoxResult, LoxValue},
    stmt::{Stmt, StmtResult},
    token::{Token, TokenType},
};
use float_cmp::{ApproxEq, F64Margin};
use std::{
    cell::RefCell,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

pub struct Interpreter<'a> {
    pub environment: Rc<RefCell<Environment>>,
    logger: &'a Rc<RefCell<LoggerImpl<'a>>>,
}

fn init_globals() -> Environment {
    let mut globals = Environment::default();
    globals.declare(
        &String::from("clock"),
        Function::Native(
            0,
            Box::new(|_| {
                LoxValue::from(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Could not retrieve time.")
                        .as_millis() as f64,
                )
            }),
        )
        .into(),
    );
    globals
}

impl<'a> Interpreter<'a> {
    pub fn new(logger: &'a Rc<RefCell<LoggerImpl<'a>>>) -> Self {
        let globals = Rc::new(RefCell::new(init_globals()));
        Interpreter {
            logger,
            environment: globals,
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) {
        for statement in statements {
            if let Err(error) = self.execute(statement, self.environment.clone()) {
                self.logger.borrow_mut().runtime_error(error);
            }
        }
    }

    fn execute(&mut self, stmt: &Stmt, env: Rc<RefCell<Environment>>) -> LoxResult<StmtResult> {
        match stmt {
            Stmt::Expression(expr) => {
                let value = self.evaluate(&expr, &mut env.borrow_mut())?;
                self.logger.borrow_mut().println_repl(format!("{}", value));
                Ok(LoxValue::Unit.into())
            }
            Stmt::Print(expr) => {
                let value = self.evaluate(&expr, &mut env.borrow_mut())?;
                self.logger.borrow_mut().println(format!("{}", value));
                Ok(LoxValue::Unit.into())
            }
            Stmt::Let(token, initializer) => {
                let value = match initializer {
                    Some(inializer_value) => {
                        self.evaluate(&inializer_value, &mut env.borrow_mut())?
                    }
                    None => LoxValue::Nil,
                };
                env.borrow_mut().declare(&token.lexeme, value);
                Ok(LoxValue::Unit.into())
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
                        StmtResult::Value(_) => (),
                    }
                }
                Ok(LoxValue::Unit.into())
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
                    Ok(LoxValue::Unit.into())
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
                        StmtResult::Value(_) => (),
                    }
                }
                Ok(LoxValue::Unit.into())
            }
            Stmt::Break(token) => {
                if env.borrow_mut().is_inside_loop() {
                    Ok(StmtResult::Break)
                } else {
                    Err(error(&token, "'break' must be inside a loop"))
                }
            }
            Stmt::Continue(token) => {
                if env.borrow_mut().is_inside_loop() {
                    Ok(StmtResult::Continue)
                } else {
                    Err(error(&token, "'continue' must be inside a loop"))
                }
            }
        }
    }

    fn evaluate(&mut self, expr: &Expr, env: &mut Environment) -> LoxResult<LoxValue> {
        match expr {
            Expr::Binary(left, operator, right) => {
                self.evaluate_binary_op(left, operator, right, env)
            }
            Expr::Grouping(expr) => self.evaluate(expr, env),
            Expr::Literal(literal) => Ok(literal.clone().into()),
            Expr::Unary(operator, right) => self.evaluate_unary_op(operator, right, env),
            Expr::Variable(token) => match env.get(&token)? {
                LoxValue::Undefined => {
                    Err(error(token, &format!("{} is undefined!", token.lexeme)))
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
            Expr::Call(callee, paren, args) => {
                let callee = self.evaluate(callee, env)?;
                let args: LoxResult<Vec<LoxValue>> =
                    args.iter().map(|arg| self.evaluate(arg, env)).collect();
                let args = args?;
                match callee {
                    LoxValue::Function(function) => {
                        if args.len() > function.arity() {
                            Err(error(
                                paren,
                                &format!(
                                    "Expected {} arguments but got {}",
                                    function.arity(),
                                    args.len()
                                ),
                            ))
                        } else {
                            Ok(function.call(self, &args))
                        }
                    }
                    _ => Err(error(paren, "Can only call functions and classes")),
                }
            }
        }
    }

    fn evaluate_unary_op(
        &mut self,
        operator: &Token,
        right: &Expr,
        env: &mut Environment,
    ) -> LoxResult<LoxValue> {
        let right = self.evaluate(&right, env)?;

        match operator.token_type {
            TokenType::BANG => Ok(LoxValue::Boolean(!right.is_truthy())),
            TokenType::MINUS => match right {
                LoxValue::Number(value) => Ok(LoxValue::Number(-value)),
                _ => Err(error(operator, "Operand must be a number")),
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
    ) -> LoxResult<LoxValue> {
        let left = self.evaluate(&left, env)?;
        let right = self.evaluate(&right, env)?;

        match (&operator.token_type, (&left, &right)) {
            (TokenType::MINUS, (LoxValue::Number(left), LoxValue::Number(right))) => {
                Ok(LoxValue::Number(left - right))
            }
            (TokenType::SLASH, (LoxValue::Number(left), LoxValue::Number(right))) => {
                if right.approx_eq(0.0, F64Margin::default()) {
                    Err(error(operator, "Division by zero"))
                } else {
                    Ok(LoxValue::Number(left / right))
                }
            }
            (TokenType::STAR, (LoxValue::Number(left), LoxValue::Number(right))) => {
                Ok(LoxValue::Number(left * right))
            }
            (TokenType::PLUS, (LoxValue::Number(left), LoxValue::Number(right))) => {
                Ok(LoxValue::Number(left + right))
            }
            (TokenType::PLUS, (LoxValue::String(left), LoxValue::String(right))) => {
                Ok(LoxValue::String(format!("{}{}", left, right)))
            }
            (TokenType::PLUS, (LoxValue::String(left), LoxValue::Number(right))) => {
                Ok(LoxValue::String(format!("{}{}", left, right)))
            }
            (TokenType::PLUS, _) => Err(error(
                operator,
                "Operands must be two numbers or two strings",
            )),
            (TokenType::GREATER, (LoxValue::Number(left), LoxValue::Number(right))) => {
                Ok(LoxValue::Boolean(left > right))
            }
            (TokenType::GREATER_EQUAL, (LoxValue::Number(left), LoxValue::Number(right))) => {
                Ok(LoxValue::Boolean(left >= right))
            }
            (TokenType::LESS, (LoxValue::Number(left), LoxValue::Number(right))) => {
                Ok(LoxValue::Boolean(left < right))
            }
            (TokenType::LESS_EQUAL, (LoxValue::Number(left), LoxValue::Number(right))) => {
                Ok(LoxValue::Boolean(left <= right))
            }
            (TokenType::BANG_EQUAL, _) => Ok(LoxValue::Boolean(!left.is_equal(right))),
            (TokenType::EQUAL_EQUAL, _) => Ok(LoxValue::Boolean(left.is_equal(right))),
            (_, _) => error_number_operand(operator),
        }
    }
}

fn error(token: &Token, message: &str) -> LoxError {
    LoxError::Runtime(ErrorData::new(token.clone(), String::from(message)))
}

fn error_number_operand(token: &Token) -> LoxResult<LoxValue> {
    Err(LoxError::Runtime(ErrorData::new(
        token.clone(),
        String::from("Operands must be numbers"),
    )))
}
