use crate::{interpreter::RuntimeError, lox::LoxValue, token::Token};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Default, Clone)]
pub struct EnvironmentData {
    pub is_loop: bool,
    pub enclosing: Option<Environment>,
    values: HashMap<String, LoxValue>,
}

#[derive(Default, Clone)]
pub struct Environment {
    pub data: Rc<RefCell<EnvironmentData>>,
}

impl Environment {
    pub fn new(enclosing: &Environment) -> Self {
        Environment {
            data: Rc::new(RefCell::new(EnvironmentData {
                values: Default::default(),
                enclosing: Some(enclosing.clone()),
                is_loop: enclosing.data.borrow().is_loop,
            })),
        }
    }

    pub fn is_inside_loop(&self) -> bool {
        if self.data.borrow().is_loop {
            true
        } else {
            match self.data.borrow().enclosing {
                Some(ref enclosing) => enclosing.is_inside_loop(),
                None => false,
            }
        }
    }

    pub fn is_enclosing_loop(&self) -> bool {
        match self.data.borrow().enclosing {
            Some(ref enclosing) => enclosing.data.borrow().is_loop,
            None => false,
        }
    }

    pub fn declare(&self, name: &String, value: LoxValue) {
        self.data.borrow_mut().values.insert(name.clone(), value);
    }

    pub fn get(&self, token: &Token) -> Result<LoxValue, RuntimeError> {
        let data = self.data.borrow();
        match data.values.get(token.lexeme.as_str()) {
            Some(value) => Ok(value.clone()),
            None => match data.enclosing {
                Some(ref enclosing) => enclosing.get(token),
                None => Err(RuntimeError(
                    token.clone(),
                    format!("Undeclared variable '{}'", token.lexeme),
                )),
            },
        }
    }

    pub fn assign(&self, token: &Token, value: LoxValue) -> Result<LoxValue, RuntimeError> {
        let mut data = self.data.borrow_mut();

        match data.values.insert(token.lexeme.clone(), value.clone()) {
            Some(_) => Ok(value),
            None => match data.enclosing {
                Some(ref enclosing) => enclosing.assign(token, value),
                None => Err(RuntimeError(
                    token.clone(),
                    format!("Undeclared variable '{}'", token.lexeme),
                )),
            },
        }
    }
}
