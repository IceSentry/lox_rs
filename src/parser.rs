use crate::{
    expr::Expr,
    scanner::{Token, TokenType},
    stmt::Stmt,
    Lox,
};

pub struct ParserError(String);

pub struct Parser<'a> {
    tokens: Vec<Token>,
    current: usize,
    lox: &'a mut Lox,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, lox: &'a mut Lox) -> Self {
        Parser {
            tokens,
            current: 0,
            lox,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ()> {
        let mut statements = Vec::new();
        let mut errors = Vec::new();
        while !self.is_at_end() {
            match self.statement() {
                Ok(statement) => statements.push(statement),
                Err(error) => errors.push(error),
            }
        }
        if errors.len() > 0 {
            Err(())
        } else {
            Ok(statements)
        }
    }

    fn statement(&mut self) -> Result<Stmt, ParserError> {
        if self.match_tokens(&vec![TokenType::PRINT]) {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> Result<Stmt, ParserError> {
        let value = self.expression();
        self.consume(TokenType::SEMICOLON, "Expect ';' after value")?;
        value.and_then(|value| Ok(Stmt::Print(Box::new(value))))
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParserError> {
        let expr = self.expression();
        self.consume(TokenType::SEMICOLON, "Expect ';' after expression")?;
        expr.and_then(|expr| Ok(Stmt::Expression(Box::new(expr))))
    }

    fn expression(&mut self) -> Result<Expr, ParserError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.comparison()?;

        use TokenType::*;
        while self.match_tokens(&vec![BANG_EQUAL, EQUAL_EQUAL]) {
            let operator = self.previous().clone();
            let right = Box::new(self.comparison()?);
            expr = Expr::Binary(Box::new(expr), operator.clone(), right);
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.addition()?;

        use TokenType::*;
        while self.match_tokens(&vec![GREATER, GREATER_EQUAL, LESS, LESS_EQUAL]) {
            let operator = self.previous().clone();
            let right = Box::new(self.addition()?);
            expr = Expr::Binary(Box::new(expr), operator.clone(), right);
        }
        Ok(expr)
    }

    fn addition(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.multiplication()?;

        use TokenType::*;
        while self.match_tokens(&vec![MINUS, PLUS]) {
            let operator = self.previous().clone();
            let right = Box::new(self.multiplication()?);
            expr = Expr::Binary(Box::new(expr), operator.clone(), right);
        }
        Ok(expr)
    }

    fn multiplication(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.unary()?;

        use TokenType::*;
        while self.match_tokens(&vec![SLASH, STAR]) {
            let operator = self.previous().clone();
            let right = Box::new(self.unary()?);
            expr = Expr::Binary(Box::new(expr), operator.clone(), right);
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParserError> {
        use TokenType::*;
        if self.match_tokens(&vec![BANG, MINUS]) {
            let operator = self.previous().clone();
            let right = Box::new(self.unary()?);
            Ok(Expr::Unary(operator.clone(), right))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr, ParserError> {
        use TokenType::*;
        if self.match_tokens(&vec![FALSE, TRUE, NIL, NUMBER, STRING]) {
            match self.previous().clone().literal {
                Some(literal) => Ok(Expr::Literal(literal)),
                _ => Err(self.error("Expected literal")),
            }
        } else if self.match_tokens(&vec![LEFT_PAREN]) {
            let expr = self.expression()?;
            self.consume(RIGHT_PAREN, "Expected ')' after expression")?;
            Ok(Expr::Grouping(Box::new(expr)))
        } else {
            Err(self.error("Expected expression"))
        }
    }

    fn consume(&mut self, token_type: TokenType, error_msg: &str) -> Result<&Token, ParserError> {
        if self.check(&token_type) {
            Ok(self.advance())
        } else {
            Err(self.error(error_msg))
        }
    }

    fn synchronise(&mut self) {
        self.advance();

        use TokenType::*;
        while !self.is_at_end() {
            if self.previous().token_type == SEMICOLON {
                return;
            }

            match self.peek().token_type {
                CLASS | FUN | VAR | FOR | IF | WHILE | PRINT | RETURN => return,
                _ => self.advance(),
            };
        }
    }

    fn match_tokens(&mut self, types: &Vec<TokenType>) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            &self.peek().token_type == token_type
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn error(&mut self, message: &str) -> ParserError {
        self.error_token(&self.peek().clone(), message)
    }

    fn error_token(&mut self, token: &Token, message: &str) -> ParserError {
        let message = String::from(message);
        self.lox.error(token, message.clone());
        ParserError(message)
    }
}
