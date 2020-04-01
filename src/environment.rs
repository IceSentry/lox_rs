use crate::{interpreter::RuntimeError, lox::LoxValue, token::Token};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Default, Clone)]
pub struct Environment {
    values: HashMap<String, LoxValue>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new(enclosing: Rc<RefCell<Environment>>) -> Self {
        Environment {
            values: Default::default(),
            enclosing: Some(enclosing),
        }
    }

    pub fn define(&mut self, name: String, value: LoxValue) {
        self.values.insert(name, value);
    }

    pub fn get(&self, token: &Token) -> Result<LoxValue, RuntimeError> {
        match self.values.get(token.lexeme.as_str()) {
            Some(value) => Ok(value.clone()),
            None => match &self.enclosing {
                Some(enclosing) => enclosing.borrow().get(token),
                None => Err(RuntimeError(
                    token.clone(),
                    format!("Undefined variable '{}'", token.lexeme),
                )),
            },
        }
    }

    pub fn assign(&mut self, token: Token, value: LoxValue) -> Result<LoxValue, RuntimeError> {
        match self.values.insert(token.lexeme.clone(), value.clone()) {
            Some(_) => Ok(value),
            None => match &self.enclosing {
                Some(enclosing) => enclosing.borrow_mut().assign(token, value),
                None => Err(RuntimeError(
                    token.clone(),
                    format!("Undefined variable '{}'", token.lexeme),
                )),
            },
        }
    }
}
