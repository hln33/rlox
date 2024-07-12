use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    function::{Callable, Function},
    interpreter::Interpreter,
    scanner::Token,
    value::Value,
    Exception, RuntimeError,
};

#[derive(Clone, Debug)]
pub struct Class {
    name: String,
    methods: HashMap<String, Function>,
}

impl Class {
    pub fn new(name: String, methods: HashMap<String, Function>) -> Class {
        Class { name, methods }
    }

    fn find_method(&self, name: &str) -> Option<Value> {
        match self.methods.get(name) {
            Some(method) => Some(Value::Function(method.clone())),
            None => None,
        }
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
        Ok(Value::ClassInstance(instance.clone()))
    }
}

pub type ClassInstanceRef = Rc<RefCell<ClassInstance>>;

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
    pub fn new(class: Class) -> ClassInstanceRef {
        Rc::new(RefCell::new(ClassInstance {
            class,
            fields: HashMap::new(),
        }))
    }

    pub fn get(&self, name: &Token) -> Result<Value, Exception> {
        if let Some(field) = self.fields.get(&name.lexeme) {
            return Ok(field.clone());
        }

        if let Some(Value::Function(method)) = self.class.find_method(&name.lexeme) {
            let bound_method = method.bind(Rc::new(RefCell::new(self.clone())));
            return Ok(Value::Function(bound_method));
        }

        Exception::runtime_error(name.clone(), format!("Undefined property {}.", name.lexeme))
    }

    pub fn set(&mut self, name: &Token, value: Value) {
        self.fields.insert(name.lexeme.clone(), value);
    }
}
