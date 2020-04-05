use std::fmt::*;

#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Number(f64),
    FALSE,
    TRUE,
    Nil, // TODO remove Nil when Option<LoxValue> exists
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Literal::Number(value) => write!(f, "{}", value),
            Literal::String(value) => write!(f, "\"{}\"", value),
            Literal::FALSE => write!(f, "false"),
            Literal::TRUE => write!(f, "true"),
            Literal::Nil => write!(f, "nil"),
        }
    }
}
