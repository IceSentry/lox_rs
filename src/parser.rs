use crate::{
    expr::Expr,
    logger::Logger,
    stmt::Stmt,
    token::{Token, TokenType},
};

pub struct ParserError(String);

pub struct Parser<'a> {
    tokens: Vec<Token>,
    current: usize,
    logger: &'a mut dyn Logger,
}

/// This parses expressions and statements according to this grammar:
///
/// program         -> declaration* EOF ;
///
/// declaration     -> let_decl
///                  | statement ;
///
/// let_decl        -> "let" IDENTIFIER ( "=" expression )? ";" ;
///
/// statement       -> print_stmt
///                  | block
///                  | if_stmt
///                  | while_stmt
///                  | expr_stmt ;
///
/// print_stmt      -> "print" expression ";" ;
/// block           -> "{" declaration* "}" ;
/// if_stmt         -> "if" expression "{" statement "}" ( "else" "{" statement "}" )? ;
/// while_stmt      -> "while" "(" expression ")" statement ;
/// expr_stmt       -> expression ";" ;
///
/// expression      -> assignment ;
///
/// assignment      -> IDENTIFIER "=" assignment
///                  | logic_or ;
///
/// logic_or        -> logic_and ( "or" logic_and)* ;
/// logic_and       -> equality ( "and" equality)* ;
///
/// equality        -> comparison ( ( "!=" | "==" ) comparison )* ;
///
/// comparison      -> addition ( ( ">" | ">=" | "<" | "<=" ) addition )* ;
///
/// addition        -> multiplication ( ( "-" | "+" ) multiplication )* ;
///
/// multiplication  -> unary ( ( "/" | "*" ) unary )* ;
///
/// unary           -> ( "!" | "-" ) unary
///                  | primary ;
///
/// primary         -> "true" | "false" | "nil"
///                  | NUMBER | STRING
///                  | "(" expression ")"
///                  | IDENTIFIER ;
impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, logger: &'a mut dyn Logger) -> Self {
        Parser {
            tokens,
            current: 0,
            logger,
        }
    }

    /// program -> declaration* EOF ;
    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParserError> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration())
        }
        statements.into_iter().collect()
    }

    /// declaration -> let_decl
    ///              | statement ;
    fn declaration(&mut self) -> Result<Stmt, ParserError> {
        let result = if self.match_tokens(&vec![TokenType::LET]) {
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
    fn let_declaration(&mut self) -> Result<Stmt, ParserError> {
        let name = self
            .consume(TokenType::IDENTIFIER, "Expected variable name")?
            .clone();

        let initializer = if self.match_tokens(&vec![TokenType::EQUAL]) {
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

    /// statement  -> print_stmt
    ///             | block
    ///             | if_stmt
    ///             | expr_stmt ;
    fn statement(&mut self) -> Result<Stmt, ParserError> {
        if self.match_tokens(&vec![TokenType::PRINT]) {
            self.print_statement()
        } else if self.match_tokens(&vec![TokenType::LEFT_BRACE]) {
            Ok(Stmt::Block(self.block()?))
        } else if self.match_tokens(&vec![TokenType::IF]) {
            self.if_statement()
        } else if self.match_tokens(&vec![TokenType::WHILE]) {
            self.while_statement()
        } else {
            self.expression_statement()
        }
    }

    // TODO use "while" expression block instead of "while" "(" expression ")" statement ;
    fn while_statement(&mut self) -> Result<Stmt, ParserError> {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after condition.")?;
        let body = self.statement()?;
        Ok(Stmt::While(condition, Box::new(body)))
    }

    /// print_stmt -> "print" expression ";" ;
    fn print_statement(&mut self) -> Result<Stmt, ParserError> {
        // TODO remove this when we have a standard library
        let value = self.expression();
        self.consume(TokenType::SEMICOLON, "Expect ';' after value")?;
        value.and_then(|value| Ok(Stmt::Print(value)))
    }

    /// block -> "{" declaration* "}" ;
    fn block(&mut self) -> Result<Vec<Box<Stmt>>, ParserError> {
        let mut statements = Vec::new();
        while !self.check(&TokenType::RIGHT_BRACE) && !self.is_at_end() {
            statements.push(Box::new(self.declaration()?));
        }
        self.consume(TokenType::RIGHT_BRACE, "Expected '}' after block")?;
        Ok(statements)
    }

    /// if_stmt -> "if" expression block ( "else" block )? ;
    fn if_statement(&mut self) -> Result<Stmt, ParserError> {
        let condition = self.expression()?;
        self.consume(TokenType::LEFT_BRACE, "Expected '{' after condition")?;

        let then_branch = Stmt::Block(self.block()?);
        println!("parsed then_branch\n{}", then_branch);

        let else_branch = if self.match_tokens(&vec![TokenType::ELSE]) {
            println!("parsed ELSE");
            let block = self.statement()?;
            println!("parsed else_branch\n{}", block);
            Some(Box::new(block))
        } else {
            None
        };

        Ok(Stmt::If(condition, Box::new(then_branch), else_branch))
    }

    /// expr_stmt  -> expression ";"
    fn expression_statement(&mut self) -> Result<Stmt, ParserError> {
        // TODO support expression with no ;
        let expr = self.expression();
        self.consume(TokenType::SEMICOLON, "Expect ';' after expression")?;
        expr.and_then(|expr| Ok(Stmt::Expression(expr)))
    }

    /// expression -> assignment ;
    fn expression(&mut self) -> Result<Expr, ParserError> {
        self.assignment()
    }

    /// assignment -> IDENTIFIER "=" assignment
    ///             | logic_or ;
    fn assignment(&mut self) -> Result<Expr, ParserError> {
        let expr = self.logic_or();

        if self.match_tokens(&vec![TokenType::EQUAL]) {
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
    fn logic_or(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.logic_and()?;
        while self.match_tokens(&vec![TokenType::OR]) {
            let operator = self.previous().clone();
            let right = self.logic_and()?;
            expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    /// logic_and -> equality ( "and" equality )* ;
    fn logic_and(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.equality()?;
        while self.match_tokens(&vec![TokenType::AND]) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    /// equality -> comparison ( ( "!=" | "==" ) comparison )* ;
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

    /// comparison -> addition ( ( ">" | ">=" | "<" | "<=" ) addition )* ;
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

    /// addition -> multiplication ( ( "-" | "+" ) multiplication )* ;
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

    /// multiplication -> unary ( ( "/" | "*" ) unary )* ;
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

    /// unary -> ( "!" | "-" ) unary
    ///        | primary ;
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

    /// primary -> "true" | "false" | "nil"
    ///          | NUMBER | STRING
    ///          | "(" expression ")"
    ///          | IDENTIFIER ;
    fn primary(&mut self) -> Result<Expr, ParserError> {
        use TokenType::*;
        if self.match_tokens(&vec![FALSE, TRUE, NIL, NUMBER, STRING]) {
            match self.previous().clone().literal {
                Some(literal) => Ok(Expr::Literal(literal)),
                _ => Err(self.error("Expected literal")),
            }
        } else if self.match_tokens(&vec![IDENTIFIER]) {
            Ok(Expr::Variable(self.previous().clone()))
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
                CLASS | FUN | LET | FOR | IF | WHILE | PRINT | RETURN => return,
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
        self.logger.error(token, message.clone());
        ParserError(message)
    }
}
