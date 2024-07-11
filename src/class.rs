use std::{collections::HashMap, fmt::Display};

use crate::{
    function::Callable, interpreter::Interpreter, scanner::Token, value::Value, Exception,
    RuntimeError,
};

#[derive(Clone, Debug)]
pub struct Class {
    name: String,
}

impl Class {
    pub fn new(name: String) -> Class {
        Class { name }
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

// class constructor
impl Callable for Class {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, Exception> {
        let instance = ClassInstance::new(self.clone());
        Ok(Value::ClassInstance(instance))
    }
}

#[derive(Clone, Debug)]
pub struct ClassInstance {
    class: Class,
    fields: HashMap<String, Value>,
}

impl Display for ClassInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class)
    }
}

impl ClassInstance {
    pub fn new(class: Class) -> ClassInstance {
        ClassInstance {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> Result<Value, Exception> {
        if let Some(field) = self.fields.get(&name.lexeme) {
            return Ok(field.clone());
        }

        Exception::runtime_error(name.clone(), format!("Undefined property {}.", name.lexeme))
    }

    pub fn set(&mut self, name: &Token, value: Value) {
        self.fields.insert(name.lexeme.clone(), value);
    }
}
