use crate::interpreter::Value;
use std::{collections::HashMap, cell::RefCell, rc::Rc};

use super::InterpretError;

#[derive(Clone, Debug)]
pub struct Environment {
    pub enclosing: Option<Rc<RefCell<Environment>>>,
    pub values: HashMap<String, Value>,
}

impl Environment {
    pub fn global() -> Self {
        Self {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn new(enclosing: Rc<RefCell<Environment>>) -> Self {
        Self {
            enclosing: Some(enclosing),
            values: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if self.values.contains_key(name) {
            self.values.get(name).cloned()
        } else {
            match &self.enclosing {
                Some(enclosing) => enclosing.as_ref().borrow().get(name),
                None => None,
            }
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: String, value: Value) -> Result<(), InterpretError> {
        if self.values.contains_key(&name) {
            self.values.insert(name, value);
            return Ok(());
        }

        if let Some(ref mut enclosing) = self.enclosing {
            return enclosing.as_ref().borrow_mut().assign(name, value);
        }

        Err(InterpretError::new(
            String::from("Undefined variable '"),
            value.token,
        ))
    }
}
