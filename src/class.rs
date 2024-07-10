use std::fmt::Display;

use crate::{function::Callable, interpreter::Interpreter, value::Value, Exception};

#[derive(Clone, Debug)]
pub struct Class {
    name: String,
}

impl Class {
    pub fn new(name: String) -> Class {
        Class { name }
    }
}

// class constructor
impl Callable for Class {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, Exception> {
        let instance = ClassInstance {
            class: self.clone(),
        };
        Ok(Value::ClassInstance(instance))
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Clone, Debug)]
pub struct ClassInstance {
    class: Class,
}

impl Display for ClassInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class)
    }
}
