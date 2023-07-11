use std::collections::HashMap;
use crate::interpreter::Value;

#[derive(Clone)]
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

    pub fn insert(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }
}
