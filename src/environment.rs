use crate::{scanner::Token, value::Value, RuntimeError};
use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    rc::Rc,
};

pub type EnvRef = Rc<RefCell<Environment>>;

pub struct Environment {
    enclosing: Option<EnvRef>,
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new_global() -> EnvRef {
        Rc::new(RefCell::new(Environment {
            enclosing: None,
            values: HashMap::new(),
        }))
    }

    pub fn new_local(enclosing: EnvRef) -> EnvRef {
        Rc::new(RefCell::new(Environment {
            enclosing: Some(enclosing),
            values: HashMap::new(),
        }))
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Value, RuntimeError> {
        if let Some(value) = self.values.get(&name.lexeme) {
            return Ok(value.clone());
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow().get(name);
        }

        Err(RuntimeError {
            token: name.clone(),
            message: format!("Undefined variable {}.", name.lexeme),
        })
    }

    pub fn assign(&mut self, name: &Token, value: Value) -> Result<(), RuntimeError> {
        if let Entry::Occupied(mut e) = self.values.entry(name.lexeme.clone()) {
            e.insert(value);
            return Ok(());
        }

        if let Some(enclosing) = &mut self.enclosing {
            return enclosing.borrow_mut().assign(name, value);
        }

        Err(RuntimeError {
            token: name.clone(),
            message: format!("Undefined variable {}.", name.lexeme),
        })
    }
}
