use std::fmt::*;

#[derive(Debug, Clone)]
pub enum Literal {
    Identifier(String),
    String(String),
    Number(f64),
    FALSE,
    TRUE,
    Nil,
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Literal::Number(value) => write!(f, "{}", value),
            Literal::String(value) => write!(f, "{}", value),
            Literal::Identifier(identifier) => write!(f, "{}", identifier),
            Literal::FALSE => write!(f, "false"),
            Literal::TRUE => write!(f, "true"),
            Literal::Nil => write!(f, "nil"),
        }
    }
}
