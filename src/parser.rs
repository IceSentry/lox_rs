use crate::{
    expr::Expr,
    literal::Literal,
    logger::{Logger, LoggerImpl},
    lox::{ErrorData, LoxError, LoxResult},
    stmt::Stmt,
    token::{Token, TokenType},
};
use std::{cell::RefCell, rc::Rc};

pub struct Parser<'a> {
    tokens: Vec<Token>,
    current: usize,
    logger: &'a Rc<RefCell<LoggerImpl<'a>>>,
}

macro_rules! match_tokens {
    ( $sel:ident, $( $x:expr ),* ) => {
        if $( $sel.check_token($x) )||* {
            $sel.advance();
            true
        } else {
            false
        }
    };
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, logger: &'a Rc<RefCell<LoggerImpl<'a>>>) -> Self {
        Parser {
            tokens,
            current: 0,
            logger,
        }
    }

    /// program -> declaration* EOF ;
    pub fn parse(&mut self) -> LoxResult<Vec<Stmt>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            let stmt = self.declaration();
            statements.push(stmt)
        }
        statements.into_iter().collect()
    }

    /// declaration -> let_decl
    ///              | statement ;
    fn declaration(&mut self) -> LoxResult<Stmt> {
        let result = if match_tokens!(self, TokenType::LET) {
            self.let_declaration()
        } else {
            self.statement()
        };

        if result.is_err() {
            self.synchronise();
        }
        result
    }

    /// let_decl -> "let" IDENTIFIER ( "=" expression )? ";" ;
    fn let_declaration(&mut self) -> LoxResult<Stmt> {
        let name = self
            .consume(TokenType::IDENTIFIER, "Expected variable name")?
            .clone();

        let initializer = if match_tokens!(self, TokenType::EQUAL) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            TokenType::SEMICOLON,
            "Expected ';' after variable declaration",
        )?;
        Ok(Stmt::Let(name, initializer))
    }

    /// statement -> expr_stmt
    ///            | for_stmt
    ///            | if_stmt
    ///            | print_stmt
    ///            | while_stmt
    ///            | break
    ///            | continue
    ///            | block ;
    fn statement(&mut self) -> LoxResult<Stmt> {
        if match_tokens!(self, TokenType::FOR) {
            self.for_statement()
        } else if match_tokens!(self, TokenType::IF) {
            self.if_statement()
        } else if match_tokens!(self, TokenType::PRINT) {
            self.print_statement()
        } else if match_tokens!(self, TokenType::WHILE) {
            self.while_statement()
        } else if match_tokens!(self, TokenType::LOOP) {
            self.loop_statement()
        } else if match_tokens!(self, TokenType::LEFT_BRACE) {
            Ok(Stmt::Block(self.block()?))
        } else if match_tokens!(self, TokenType::BREAK) {
            self.consume(TokenType::SEMICOLON, "Expected ';' after break")?;
            Ok(Stmt::Break(self.previous().clone()))
        } else if match_tokens!(self, TokenType::CONTINUE) {
            self.consume(TokenType::SEMICOLON, "Expected ';' after continue")?;
            Ok(Stmt::Continue(self.previous().clone()))
        } else {
            self.expression_statement()
        }
    }

    /// for_stmt -> "for" "(" ( let_decl | expr_stmt | ";" )
    ///                       expression? ";"
    ///                       expression? ")" statement ;
    /// TODO "for" IDENTIFIER "in" IDENTIFIER block ;
    fn for_statement(&mut self) -> LoxResult<Stmt> {
        self.consume(TokenType::LEFT_PAREN, "Expected '(' after 'for'")?;
        let initializer = if match_tokens!(self, TokenType::SEMICOLON) {
            None
        } else if match_tokens!(self, TokenType::LET) {
            Some(self.let_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let mut condition = match self.check() {
            Some(TokenType::SEMICOLON) => None,
            _ => Some(self.expression()?),
        };
        self.consume(TokenType::SEMICOLON, "Expected ';' after loop condition")?;

        let increment = match self.check() {
            Some(TokenType::RIGHT_PAREN) => None,
            _ => Some(self.expression()?),
        };
        self.consume(TokenType::RIGHT_PAREN, "Expected ')' after for clauses")?;

        let mut body = self.statement()?;
        if let Some(increment) = increment {
            body = Stmt::Block(vec![Box::new(body), Box::new(Stmt::Expression(increment))]);
        }
        if let None = condition {
            condition = Some(Expr::Literal(Literal::TRUE))
        }
        body = Stmt::While(
            condition.expect("condition should be Some() at this point"),
            Box::new(body),
        );
        if let Some(initializer) = initializer {
            body = Stmt::Block(vec![Box::new(initializer), Box::new(body)]);
        }
        Ok(body)
    }

    /// "loop" block_statement ;
    fn loop_statement(&mut self) -> LoxResult<Stmt> {
        let body = self.block_statement()?;
        Ok(Stmt::While(Expr::Literal(Literal::TRUE), Box::new(body)))
    }

    /// "while" expression block_statement ;
    fn while_statement(&mut self) -> LoxResult<Stmt> {
        let condition = self.expression()?;
        let body = self.block_statement()?;
        Ok(Stmt::While(condition, Box::new(body)))
    }

    /// print_stmt -> "print" expression ";" ;
    /// TODO remove this when we have a standard library
    fn print_statement(&mut self) -> LoxResult<Stmt> {
        let value = self.expression();
        self.consume(TokenType::SEMICOLON, "Expect ';' after value")?;
        value.and_then(|value| Ok(Stmt::Print(value)))
    }

    /// block_statement -> "{" block ;
    fn block_statement(&mut self) -> LoxResult<Stmt> {
        self.consume(TokenType::LEFT_BRACE, "Expected '{'")?;
        Ok(Stmt::Block(self.block()?))
    }

    /// block -> "{" declaration* "}" ;
    fn block(&mut self) -> LoxResult<Vec<Box<Stmt>>> {
        let mut statements = Vec::new();
        while !self.check_token(TokenType::RIGHT_BRACE) && !self.is_at_end() {
            statements.push(Box::new(self.declaration()?));
        }
        self.consume(TokenType::RIGHT_BRACE, "Expected '}' after block")?;
        Ok(statements)
    }

    /// if_stmt -> "if" expression block ( "else" block )? ;
    fn if_statement(&mut self) -> LoxResult<Stmt> {
        let condition = self.expression()?;
        let then_branch = self.block_statement()?;
        let else_branch = if match_tokens!(self, TokenType::ELSE) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If(condition, Box::new(then_branch), else_branch))
    }

    /// expr_stmt  -> expression ";"
    fn expression_statement(&mut self) -> LoxResult<Stmt> {
        // TODO support expression with no ;
        let expr = self.expression();
        self.consume(TokenType::SEMICOLON, "Expect ';' after expression")?;
        expr.and_then(|expr| Ok(Stmt::Expression(expr)))
    }

    /// expression -> assignment ;
    fn expression(&mut self) -> LoxResult<Expr> {
        self.assignment()
    }

    /// assignment -> IDENTIFIER "=" assignment
    ///             | logic_or ;
    fn assignment(&mut self) -> LoxResult<Expr> {
        let expr = self.logic_or();

        if match_tokens!(self, TokenType::EQUAL) {
            let equals = self.previous().clone();
            match expr {
                Ok(Expr::Variable(token)) => {
                    let value = self.assignment()?;
                    Ok(Expr::Assign(token, Box::new(value)))
                }
                _ => Err(self.error_token(&equals, "Invalid assignment target")),
            }
        } else {
            expr
        }
    }

    /// logic_or -> logic_and ( "or" logic_and )* ;
    fn logic_or(&mut self) -> LoxResult<Expr> {
        let mut expr = self.logic_and()?;
        while match_tokens!(self, TokenType::OR) {
            let operator = self.previous().clone();
            let right = self.logic_and()?;
            expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    /// logic_and -> equality ( "and" equality )* ;
    fn logic_and(&mut self) -> LoxResult<Expr> {
        let mut expr = self.equality()?;
        while match_tokens!(self, TokenType::AND) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    /// equality -> comparison ( ( "!=" | "==" ) comparison )* ;
    fn equality(&mut self) -> LoxResult<Expr> {
        let mut expr = self.comparison()?;
        use TokenType::*;
        while match_tokens!(self, BANG_EQUAL, EQUAL_EQUAL) {
            let operator = self.previous().clone();
            let right = Box::new(self.comparison()?);
            expr = Expr::Binary(Box::new(expr), operator.clone(), right);
        }
        Ok(expr)
    }

    /// comparison -> addition ( ( ">" | ">=" | "<" | "<=" ) addition )* ;
    fn comparison(&mut self) -> LoxResult<Expr> {
        let mut expr = self.addition()?;
        use TokenType::*;
        while match_tokens!(self, GREATER, GREATER_EQUAL, LESS, LESS_EQUAL) {
            let operator = self.previous().clone();
            let right = Box::new(self.addition()?);
            expr = Expr::Binary(Box::new(expr), operator.clone(), right);
        }
        Ok(expr)
    }

    /// addition -> multiplication ( ( "-" | "+" ) multiplication )* ;
    fn addition(&mut self) -> LoxResult<Expr> {
        let mut expr = self.multiplication()?;
        use TokenType::*;
        while match_tokens!(self, MINUS, PLUS) {
            let operator = self.previous().clone();
            let right = Box::new(self.multiplication()?);
            expr = Expr::Binary(Box::new(expr), operator.clone(), right);
        }
        Ok(expr)
    }

    /// multiplication -> unary ( ( "/" | "*" ) unary )* ;
    fn multiplication(&mut self) -> LoxResult<Expr> {
        let mut expr = self.unary()?;
        use TokenType::*;
        while match_tokens!(self, SLASH, STAR) {
            let operator = self.previous().clone();
            let right = Box::new(self.unary()?);
            expr = Expr::Binary(Box::new(expr), operator.clone(), right);
        }
        Ok(expr)
    }

    /// unary -> ( "!" | "-" ) unary
    ///        | function_call ;
    fn unary(&mut self) -> LoxResult<Expr> {
        use TokenType::*;
        if match_tokens!(self, BANG, MINUS) {
            let operator = self.previous().clone();
            let right = Box::new(self.unary()?);
            Ok(Expr::Unary(operator.clone(), right))
        } else {
            self.function_call()
        }
    }

    /// function_call -> primary ( "(" arguments? ")" )* ;
    /// arguments -> expression ( "," expression )*;
    fn function_call(&mut self) -> LoxResult<Expr> {
        let mut expr = self.primary()?;
        loop {
            if match_tokens!(self, TokenType::LEFT_PAREN) {
                expr = self.finish_function_call(expr)?;
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_function_call(&mut self, callee: Expr) -> LoxResult<Expr> {
        let mut args = Vec::new();
        if !self.check_token(TokenType::RIGHT_PAREN) {
            loop {
                if args.len() >= 255 {
                    self.error("Cannot have more than 255 arguments");
                }
                args.push(self.expression()?);
                if !match_tokens!(self, TokenType::COMMA) {
                    break;
                }
            }
        }
        let paren = self.consume(TokenType::RIGHT_PAREN, "Expected ')' after arguments")?;
        Ok(Expr::FunctionCall(
            Box::new(callee),
            paren.clone(),
            Box::new(args),
        ))
    }

    /// primary -> "true" | "false" | "nil"
    ///          | NUMBER | STRING
    ///          | "(" expression ")"
    ///          | IDENTIFIER ;
    fn primary(&mut self) -> LoxResult<Expr> {
        use TokenType::*;
        if match_tokens!(self, FALSE, TRUE, NIL, NUMBER, STRING) {
            match self.previous().clone().literal {
                Some(literal) => Ok(Expr::Literal(literal)),
                _ => Err(self.error("Expected literal")),
            }
        } else if match_tokens!(self, IDENTIFIER) {
            Ok(Expr::Variable(self.previous().clone()))
        } else if match_tokens!(self, LEFT_PAREN) {
            let expr = self.expression()?;
            self.consume(RIGHT_PAREN, "Expected ')' after expression")?;
            Ok(Expr::Grouping(Box::new(expr)))
        } else {
            Err(self.error("Expected expression"))
        }
    }

    fn consume(&mut self, token_type: TokenType, error_msg: &str) -> LoxResult<&Token> {
        if let Some(check_token) = self.check() {
            if check_token == token_type {
                return Ok(self.advance());
            }
        }
        Err(self.error(error_msg))
    }

    fn synchronise(&mut self) {
        self.advance();

        use TokenType::*;
        while !self.is_at_end() {
            if self.previous().token_type == SEMICOLON {
                return;
            }

            match self.peek().token_type {
                CLASS | FUN | LET | FOR | IF | WHILE | PRINT | RETURN | LOOP => return,
                _ => self.advance(),
            };
        }
    }

    fn check(&self) -> Option<TokenType> {
        if self.is_at_end() {
            None
        } else {
            Some(self.peek().token_type.clone())
        }
    }

    fn check_token(&self, token_type: TokenType) -> bool {
        if let Some(check_token_type) = self.check() {
            check_token_type == token_type
        } else {
            false
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

    fn error(&mut self, message: &str) -> LoxError {
        self.error_token(&self.peek().clone(), message)
    }

    fn error_token(&mut self, token: &Token, message: &str) -> LoxError {
        let message = String::from(message);
        self.logger.borrow_mut().error(token, message.clone());
        LoxError::Parser(ErrorData::new(token.clone(), message))
    }
}
