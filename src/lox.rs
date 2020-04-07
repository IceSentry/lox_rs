use crate::{
    function::Function,
    interpreter::Interpreter,
    literal::Literal,
    logger::{Logger, LoggerImpl},
    parser::Parser,
    scanner::Scanner,
    token::Token,
};

use derive_new::new;
use float_cmp::{ApproxEq, F64Margin};
use std::{cell::RefCell, fmt, rc::Rc};

#[derive(Debug, Clone)]
pub enum LoxValue {
    Nil,       // TODO implement Option<T> and remove nil
    Undefined, // This is used as a flag, there are no corresponding literal
    Number(f64),
    Boolean(bool),
    String(String),
    Function(Function),
    Unit,
}

impl From<f64> for LoxValue {
    fn from(value: f64) -> Self {
        LoxValue::Number(value)
    }
}

impl From<String> for LoxValue {
    fn from(value: String) -> Self {
        LoxValue::String(value)
    }
}

impl From<bool> for LoxValue {
    fn from(value: bool) -> Self {
        LoxValue::Boolean(value)
    }
}

impl From<()> for LoxValue {
    fn from(_value: ()) -> Self {
        LoxValue::Unit
    }
}

impl From<Literal> for LoxValue {
    fn from(literal: Literal) -> Self {
        match literal {
            Literal::String(value) => LoxValue::from(value),
            Literal::Number(value) => LoxValue::from(value),
            Literal::FALSE => LoxValue::from(false),
            Literal::TRUE => LoxValue::from(true),
            Literal::Nil => LoxValue::Nil,
        }
    }
}

impl From<Function> for LoxValue {
    fn from(function: Function) -> Self {
        LoxValue::Function(function)
    }
}

impl LoxValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            LoxValue::Nil | LoxValue::Undefined => false,
            LoxValue::Boolean(value) => *value,
            _ => true,
        }
    }

    pub fn is_equal(&self, other: LoxValue) -> bool {
        match (self, other) {
            (LoxValue::Nil, LoxValue::Nil) => true,
            (LoxValue::Number(a), LoxValue::Number(b)) => a.approx_eq(b, F64Margin::default()),
            (LoxValue::String(ref a), LoxValue::String(ref b)) => *a == *b,
            (LoxValue::Boolean(a), LoxValue::Boolean(b)) => *a == b,
            _ => false, // no type coercion
        }
    }
}

impl fmt::Display for LoxValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoxValue::Nil => write!(f, "nil"),
            LoxValue::Undefined => write!(f, "undefined"),
            LoxValue::Number(value) => write!(f, "{}", value),
            LoxValue::Boolean(value) => write!(f, "{}", value),
            LoxValue::String(value) => write!(f, "{}", value),
            LoxValue::Function(function) => function.fmt(f),
            LoxValue::Unit => write!(f, "()"),
        }
    }
}

pub struct Lox<'a> {
    pub logger: &'a Rc<RefCell<LoggerImpl<'a>>>,
    pub interpreter: Interpreter<'a>,
    print_ast: bool,
}

#[derive(new)]
pub struct ErrorData {
    pub token: Token,
    pub message: String,
}

pub enum LoxError {
    Parser(ErrorData),
    Runtime(ErrorData),
    Panic(ErrorData),
}

pub type LoxResult<T> = std::result::Result<T, LoxError>;

impl<'a> Lox<'a> {
    pub fn new(logger: &'a Rc<RefCell<LoggerImpl<'a>>>, print_ast: bool) -> Self {
        Lox {
            logger,
            interpreter: Interpreter::new(logger),
            print_ast,
        }
    }

    pub fn run(&mut self, source: &str) -> LoxResult<()> {
        let mut scanner = Scanner::new(self.logger, String::from(source));
        let tokens = scanner.scan_tokens();
        let mut parser = Parser::new(tokens.to_vec(), self.logger);
        let statements = parser.parse()?;
        if self.print_ast {
            // TODO print to <file>.ast.lox
            self.logger
                .borrow_mut()
                .println(format!("{:?}", self.interpreter.environment));
            for statement in statements {
                self.logger
                    .borrow_mut()
                    .println(format!("{:#?}", statement));
            }
        } else {
            self.interpreter.interpret(&statements);
        }
        Ok(())
    }
}
