use std::fmt::Display;

use crate::interpreter::Interpreter;

#[derive(Clone)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    String(String),
    Function(Function),
    Nil,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s;
        match self {
            Value::Boolean(value) => s = value.to_string(),
            Value::Number(value) => {
                s = value.to_string();
                if s.ends_with(".0") {
                    s = s.strip_suffix(".0").unwrap().to_string();
                }
            }
            Value::String(value) => s = value.clone(),
            Value::Nil => s = String::from("nil"),
            Value::Function(_) => s = String::from("<fn>"),
        }

        write!(f, "{}", s)
    }
}

pub trait Callable {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Value;
}

#[derive(Clone)]
pub struct Function {
    pub arity: usize,
    pub callable: fn(&mut Interpreter, Vec<Value>) -> Value,
}

impl Callable for Function {
    fn arity(&self) -> usize {
        self.arity
    }

    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Value {
        (self.callable)(interpreter, args)
    }
}
