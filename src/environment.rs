use crate::{interpreter::InterpreterError, lox::LoxValue, token::Token};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Default, Clone)]
pub struct Environment {
    pub is_loop: bool,
    pub enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, LoxValue>,
}

impl Environment {
    pub fn new(enclosing: &Rc<RefCell<Environment>>) -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: Some(enclosing.clone()),
            is_loop: enclosing.borrow().is_loop,
        }
    }

    pub fn is_inside_loop(&self) -> bool {
        if self.is_loop {
            true
        } else {
            match self.enclosing {
                Some(ref enclosing) => enclosing.borrow().is_inside_loop(),
                None => false,
            }
        }
    }

    pub fn is_enclosing_loop(&self) -> bool {
        match self.enclosing {
            Some(ref enclosing) => enclosing.borrow().is_loop,
            None => false,
        }
    }

    pub fn declare(&mut self, name: &String, value: LoxValue) {
        self.values.insert(name.clone(), value);
    }

    pub fn get(&self, token: &Token) -> Result<LoxValue, InterpreterError> {
        match self.values.get(token.lexeme.as_str()) {
            Some(value) => Ok(value.clone()),
            None => match self.enclosing {
                Some(ref enclosing) => enclosing.borrow().get(token),
                None => Err(InterpreterError::Panic(
                    token.clone(),
                    format!("Undeclared variable '{}'", token.lexeme),
                )),
            },
        }
    }

    pub fn assign(&mut self, token: &Token, value: LoxValue) -> Result<LoxValue, InterpreterError> {
        match self.values.insert(token.lexeme.clone(), value.clone()) {
            Some(_) => Ok(value),
            None => match self.enclosing {
                Some(ref enclosing) => enclosing.borrow_mut().assign(token, value),
                None => Err(InterpreterError::Panic(
                    token.clone(),
                    format!("Undeclared variable '{}'", token.lexeme),
                )),
            },
        }
    }
}
