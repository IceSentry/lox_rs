use crate::{
    lox::{ErrorData, LoxError, LoxResult, LoxValue},
    token::Token,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

// TODO
// * put each env in a list and reference the id
// * env could have a scope type
// * add continue() and break().
//      They should unwind the env to the nearest loop scope
//      WARN This will break the desugared for loop

#[derive(Debug, Default, Clone)]
pub struct Environment {
    pub is_loop: bool, // TODO scope type: function, loop, block
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

    pub fn declare(&mut self, name: &str, value: LoxValue) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, token: &Token) -> LoxResult<LoxValue> {
        match self.values.get(token.lexeme.as_str()) {
            Some(value) => Ok(value.clone()),
            None => match self.enclosing {
                Some(ref enclosing) => enclosing.borrow().get(token),
                None => Err(LoxError::Panic(ErrorData::new(
                    token.clone(),
                    format!("Undeclared variable '{}'", token.lexeme),
                ))),
            },
        }
    }

    pub fn assign(&mut self, token: &Token, value: LoxValue) -> LoxResult<LoxValue> {
        match self.values.insert(token.lexeme.clone(), value.clone()) {
            Some(_) => Ok(value),
            None => match self.enclosing {
                Some(ref enclosing) => enclosing.borrow_mut().assign(token, value),
                None => Err(LoxError::Panic(ErrorData::new(
                    token.clone(),
                    format!("Undeclared variable '{}'", token.lexeme),
                ))),
            },
        }
    }
}
