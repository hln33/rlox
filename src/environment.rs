use crate::{scanner::Token, value::Value, Exception};
use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    rc::Rc,
};

pub type EnvRef = Rc<RefCell<Environment>>;

#[derive(Debug)]
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

    pub fn new_local(enclosing: &EnvRef) -> EnvRef {
        Rc::new(RefCell::new(Environment {
            enclosing: Some(enclosing.clone()),
            values: HashMap::new(),
        }))
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get_at(&self, distance: usize, name: &str) -> Result<Value, Exception> {
        if distance == 0 {
            return Ok(self.values.get(name).unwrap().clone());
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow().get_at(distance - 1, name);
        }

        panic!("Could not find local scope that variable belongs to!")
    }

    pub fn assign_at(&mut self, distance: usize, name: &Token, value: &Value) {
        if distance == 0 {
            self.values.insert(name.lexeme.clone(), value.clone());
            return;
        }

        if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().assign_at(distance - 1, name, value);
            return;
        }

        panic!("Could not find local scope that variable belongs to!")
    }

    pub fn get(&self, name: &Token) -> Result<Value, Exception> {
        if let Some(value) = self.values.get(&name.lexeme) {
            return Ok(value.clone());
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow().get(name);
        }

        Exception::runtime_error(name.clone(), format!("Undefined variable {}.", name.lexeme))
    }

    pub fn assign(&mut self, name: &Token, value: &Value) -> Result<(), Exception> {
        if let Entry::Occupied(mut e) = self.values.entry(name.lexeme.clone()) {
            e.insert(value.clone());
            return Ok(());
        }

        if let Some(enclosing) = &mut self.enclosing {
            return enclosing.borrow_mut().assign(name, value);
        }

        Exception::runtime_error(name.clone(), format!("Undefined variable {}.", name.lexeme))
    }
}
