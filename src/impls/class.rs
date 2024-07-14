use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    interpreter::Interpreter,
    syntax::{token::Token, value::Value},
    Exception,
};

use super::function::{Callable, Function};

#[derive(Clone, Debug)]
pub struct Class {
    name: String,
    super_class: Option<Box<Class>>,
    methods: HashMap<String, Function>,
}

impl Class {
    pub fn new(
        name: String,
        super_class: Option<Box<Class>>,
        methods: HashMap<String, Function>,
    ) -> Class {
        Class {
            name,
            super_class,
            methods,
        }
    }

    pub fn find_method(&self, name: &str) -> Option<Value> {
        self.methods
            .get(name)
            .map(|method| Value::Function(method.clone()))
            .or(self
                .super_class
                .as_ref()
                .and_then(|super_class| super_class.find_method(name)))
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
        if let Some(initializer) = self.find_method("init") {
            match initializer {
                Value::Function(initializer) => return initializer.arity(),
                _ => panic!("initializer is not a function!"),
            }
        }

        0
    }

    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, Exception> {
        let instance = ClassInstance::new(self.clone());

        if let Some(initializer) = self.find_method("init") {
            match initializer {
                Value::Function(initializer) => {
                    let _ = initializer.bind(instance.clone()).call(interpreter, args);
                }
                _ => panic!("initalizer is not a function!"),
            };
        }

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

    pub fn get(&self, name: &Token, instance_ref: ClassInstanceRef) -> Result<Value, Exception> {
        if let Some(field) = self.fields.get(&name.lexeme) {
            return Ok(field.clone());
        }

        if let Some(Value::Function(method)) = self.class.find_method(&name.lexeme) {
            let bound_method = method.bind(instance_ref.clone());
            return Ok(Value::Function(bound_method));
        }

        Exception::runtime_error(name.clone(), format!("Undefined property {}.", name.lexeme))
    }

    pub fn set(&mut self, name: &Token, value: Value) {
        self.fields.insert(name.lexeme.clone(), value);
    }
}
