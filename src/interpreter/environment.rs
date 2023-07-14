use crate::interpreter::Value;
use std::collections::HashMap;

use super::InterpretError;

#[derive(Clone, Debug)]
pub struct Environment {
    pub enclosing: Option<Box<Environment>>,
    pub values: HashMap<String, Value>,
}

impl Environment {
    pub fn global() -> Self {
        Self {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn new(enclosing: Box<Environment>) -> Self {
        Self {
            enclosing: Some(enclosing),
            values: HashMap::new(),
        }
    }

    pub fn clear_child(&mut self) {
        self.values = self.enclosing.as_ref().unwrap().values.clone();
        self.enclosing =  match self.enclosing {
            Some(ref mut enclosing) => enclosing.enclosing.take(),
            None => None,
        };
    }

    pub fn contains_key(&self, name: &str) -> bool {
        if self.values.contains_key(name) {
            true
        } else {
            match &self.enclosing {
                Some(enclosing) => enclosing.contains_key(name),
                None => false,
            }
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if self.values.contains_key(name) {
            self.values.get(name).cloned()
        } else {
            match &self.enclosing {
                Some(enclosing) => enclosing.get(name),
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
            return enclosing.assign(name, value);
        }

        Err(InterpretError {
            token: value.token,
            message: format!("Undefined variable '{}'.", name),
        })
    }
}
