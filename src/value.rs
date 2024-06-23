use std::fmt::Display;

use crate::interpreter::Interpreter;

#[derive(Clone)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    String(String),
    Class,
    Function,
    Nil,
}

pub trait Callable {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &Interpreter, args: Vec<Value>) -> Value;
}
impl Callable for Value {
    fn arity(&self) -> usize {
        todo!()
    }

    fn call(&self, interpreter: &Interpreter, args: Vec<Value>) -> Value {
        todo!()
    }
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
            Value::Class => todo!(),
            Value::Function => todo!(),
        }

        write!(f, "{}", s)
    }
}
