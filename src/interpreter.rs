use crate::{
    environment::Environment,
    expr::Expr,
    literal::Literal,
    logger::Logger,
    lox::LoxValue,
    stmt::Stmt,
    token::{Token, TokenType},
};

// TODO:
// * Take a custom logger?
// * use visitor pattern

pub struct RuntimeError(pub Token, pub String);

pub struct Interpreter<'a> {
    logger: &'a mut dyn Logger,
}

impl<'a> Interpreter<'a> {
    pub fn new(logger: &'a mut dyn Logger) -> Self {
        Interpreter { logger }
    }

    pub fn interpret(&mut self, statements: &Vec<Stmt>, env: &mut Environment) {
        for statement in statements {
            self.logger.println_debug(format!("{}", statement));
            if let Err(error) = self.execute(statement, env) {
                self.logger.runtime_error(error);
            }
        }
    }

    fn evaluate(&mut self, expr: &Expr, env: &mut Environment) -> Result<LoxValue, RuntimeError> {
        match expr {
            Expr::Binary(left, operator, right) => {
                self.evaluate_binary_op(left, operator, right, env)
            }
            Expr::Grouping(expr) => self.evaluate(expr, env),
            Expr::Literal(literal) => self.evaluate_literal(literal),
            Expr::Unary(operator, right) => self.evaluate_unary_op(operator, right, env),
            Expr::Variable(token) => match env.get(&token)? {
                LoxValue::Undefined => {
                    self.error(token, &format!("{} is undefined!", token.lexeme))
                }
                value => Ok(value),
            },
            Expr::Assign(token, value_expr) => {
                let value = self.evaluate(&value_expr, env)?;
                env.assign(token, value)
            }
            Expr::Logical(left, operator, right) => {
                let left = self.evaluate(left, env)?;
                match operator.token_type {
                    TokenType::OR => {
                        if left.is_truthy() {
                            return Ok(left);
                        }
                    }
                    _ => {
                        if !left.is_truthy() {
                            return Ok(left);
                        }
                    }
                };
                self.evaluate(right, env)
            }
        }
    }

    fn execute(&mut self, stmt: &Stmt, env: &mut Environment) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Expression(expr) => {
                let value = self.evaluate(&expr, env)?;
                self.logger.println_repl(format!("{}", value));
                Ok(())
            }
            Stmt::Print(expr) => {
                let value = self.evaluate(&expr, env)?;
                self.logger.println(format!("{}", value));
                Ok(())
            }
            Stmt::Let(token, initializer) => {
                let value = match initializer {
                    Some(inializer_value) => self.evaluate(&inializer_value, env)?,
                    None => LoxValue::Nil,
                };
                env.declare(&token.lexeme, value);
                Ok(())
            }
            Stmt::Block(statements) => {
                let mut environment = Environment::new(&env);
                for stmt in statements {
                    self.execute(stmt, &mut environment)?;
                }
                Ok(())
            }
            Stmt::If(condition, then_branch, else_branch) => {
                if self.evaluate(&condition, env)?.is_truthy() {
                    self.execute(then_branch, env)
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch, env)
                } else {
                    Ok(())
                }
            }
            Stmt::While(condition, body) => {
                while self.evaluate(&condition, env)?.is_truthy() {
                    self.execute(body, env)?;
                }
                Ok(())
            }
        }
    }

    fn evaluate_literal(&self, literal: &Literal) -> Result<LoxValue, RuntimeError> {
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
    ) -> Result<LoxValue, RuntimeError> {
        let right = self.evaluate(&right, env)?;

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
        left: &Expr,
        operator: &Token,
        right: &Expr,
        env: &mut Environment,
    ) -> Result<LoxValue, RuntimeError> {
        let left = self.evaluate(&left, env)?;
        let right = self.evaluate(&right, env)?;

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

    fn error(&self, token: &Token, message: &str) -> Result<LoxValue, RuntimeError> {
        Err(RuntimeError(token.clone(), String::from(message)))
    }

    fn error_number_operand(&self, token: &Token) -> Result<LoxValue, RuntimeError> {
        Err(RuntimeError(
            token.clone(),
            String::from("Operands must be numbers"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{logger::test::TestLogger, lox::Lox};

    fn lox_run(source: &str) -> Vec<u8> {
        let mut output = Vec::new();
        let mut logger = TestLogger::new(&mut output);
        let mut lox = Lox::new(&mut logger);
        let result = lox.run(source);
        assert!(result.is_ok());
        output.clone()
    }

    fn assert_output(source: &str, expected: &str) {
        let output = lox_run(source);
        assert_eq!(
            String::from_utf8(output).expect("Not UTF-8").trim(),
            expected,
        );
    }

    fn assert_output_list(source: &str, expected: Vec<&str>) {
        let output = lox_run(source);
        let output = String::from_utf8(output).expect("Not UTF-8");
        for (i, result) in output.split('\n').into_iter().enumerate() {
            if !result.is_empty() {
                assert_eq!(result, expected[i]);
            }
        }
    }

    #[test]
    fn test_shadowing() {
        let source = r#"
            let a = 1;
            {
                let a = a + 2;
                print a; // 3
            }
        "#;
        assert_output(source, "3");
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
                let d = "outer d";
                {
                    let a = "inner a";
                    d = "inner d";
                    print a;
                    print b;
                    print c;
                }
                print a;
                print b;
                print c;
                print d;
            }
            print a;
            print b;
            print c;
        "#;

        assert_output_list(
            source,
            vec![
                "inner a", "outer b", "global c", "outer a", "outer b", "global c", "inner d",
                "global a", "global b", "global c",
            ],
        )
    }

    #[test]
    fn test_operator_precedence() {
        let source = r#"
            print 2 + 3 * 4 * 5 - 6;
        "#;
        assert_output(source, "56");
    }

    #[test]
    fn test_if() {
        let source = r#"
            if true {
                print "then_branch"; // <--
            } else if false {
                print "else_if_branch";
            } else {
                print "else_branch";
            }
        "#;
        assert_output(source, "then_branch");

        let source = r#"
            if false {
                print "then_branch";
            } else if true {
                print "else_if_branch"; // <--
            } else {
                print "else_branch";
            }
        "#;
        assert_output(source, "else_if_branch");

        let source = r#"
            if false {
                print "then_branch";
            } else if false {
                print "else_if_branch";
            } else {
                print "else_branch"; // <--
            }
        "#;
        assert_output(source, "else_branch");
    }

    #[test]
    fn test_logical_operator() {
        let source = r#"
            print "hi" or 2; // "hi".
            print nil or "yes"; // "yes".
        "#;

        assert_output_list(source, vec!["hi", "yes"]);
    }

    #[test]
    fn test_while() {
        let source = r#"
            let i = 0;
            while (i < 5) {
                print i;
                i = i + 1;
            }
        "#;

        assert_output_list(source, vec!["0", "1", "2", "3", "4"]);
    }
}
