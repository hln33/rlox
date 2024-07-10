use std::fmt::Display;

use crate::{
    class::Class,
    function::{Function, NativeFunction},
};

#[derive(Clone, Debug)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    String(String),
    Function(Function),
    NativeFunction(NativeFunction),
    Class(Class),
    Nil,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Value::Boolean(value) => value.to_string(),
            Value::Number(value) => {
                let mut res = value.to_string();
                if res.ends_with(".0") {
                    res = res.strip_suffix(".0").unwrap().to_string();
                }
                res
            }
            Value::String(value) => value.clone(),
            Value::Nil => String::from("nil"),
            Value::Function(_) => String::from("<fn>"),
            Value::NativeFunction(_) => String::from("<native fn>"),
            Value::Class(class) => class.to_string(),
        };

        write!(f, "{}", s)
    }
}
