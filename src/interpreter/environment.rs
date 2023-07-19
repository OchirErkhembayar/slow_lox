use crate::interpreter::Value;
use std::{collections::{HashMap, hash_map}, cell::RefCell, rc::Rc};

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

    pub fn get_global(&self, name: &str) -> Option<Value> {
        let mut environment = self.clone();
        while let Some(enclosing) = environment.enclosing {
            environment = enclosing.as_ref().borrow().clone();
        }
        environment.get(0, name)
    }

    pub fn get(&self, distance: usize, name: &str) -> Option<Value> {
        if self.values.contains_key(name) {
            self.values.get(name).cloned()
        } else {
            self.ancestor(distance).as_ref().borrow().get(distance, name)
        }
    }

    fn ancestor(&self, distance: usize) -> Rc<RefCell<Environment>> {
        let mut environment = self.enclosing.as_ref().unwrap().borrow().clone();
        for _ in 0..distance {
            environment = environment.enclosing.unwrap().as_ref().borrow().clone();
        }
        Rc::new(RefCell::new(environment))
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: String, value: Value) -> Result<(), InterpretError> {
        if let hash_map::Entry::Occupied(mut entry) = self.values.entry(name.clone()) {
            entry.insert(value);
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

    pub fn assign_at(&mut self, distance: usize, name: String, value: Value) {
        self.ancestor(distance).as_ref().borrow_mut().values.insert(name, value);
    }

    pub fn assign_global(&mut self, name: String, value: Value) {
        let mut environment = self.clone();
        while let Some(enclosing) = environment.enclosing {
            environment = enclosing.as_ref().borrow().clone();
        }
        environment.values.insert(name, value);
    }
}
