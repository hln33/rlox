use crate::{interpreter::Value, scanner::Token, RuntimeError};
use std::collections::HashMap;

pub struct Environment {
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Value, RuntimeError> {
        match self.values.get(&name.lexeme) {
            Some(value) => Ok(value.clone()),
            None => Err(RuntimeError {
                token: name.clone(),
                message: format!("Undefined variable {}.", name.lexeme),
            }),
        }
    }
}
