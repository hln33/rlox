use std::fmt::Display;

use crate::{
    environment::Environment, interpreter::Interpreter, scanner::Token, stmt::Stmt, RuntimeError,
};

#[derive(Clone)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    String(String),
    Function(Function),
    NativeFunction(NativeFunction),
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
        };

        write!(f, "{}", s)
    }
}

pub trait Callable {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError>;
    fn check_arity(&self, args_len: usize, current_token: &Token) -> Result<(), RuntimeError> {
        if args_len > self.arity() {
            return Err(RuntimeError {
                token: current_token.clone(),
                message: format!("Expected {} arguments but got {}.", self.arity(), args_len),
            });
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct NativeFunction {
    pub arity: usize,
    pub callable: fn(&mut Interpreter, Vec<Value>) -> Value,
}
impl Callable for NativeFunction {
    fn arity(&self) -> usize {
        self.arity
    }

    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
        Ok((self.callable)(interpreter, args))
    }
}

#[derive(Clone)]
pub struct Function {
    pub declaration: Stmt,
}

impl Callable for Function {
    fn arity(&self) -> usize {
        if let Stmt::Function {
            name: _,
            params,
            body: _,
        } = &self.declaration
        {
            return params.len();
        }
        panic!("Function was not passed a function declaration as a statement!");
    }

    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let environment = Environment::new_local(&interpreter.globals);

        if let Stmt::Function {
            name: _,
            params,
            body,
        } = &self.declaration
        {
            for (i, param) in params.iter().enumerate() {
                environment
                    .borrow_mut()
                    .define(param.lexeme.clone(), args.get(i).unwrap().clone())
            }
            interpreter.execute_block(body, environment)?;
        }

        // will add return values later
        Ok(Value::Nil)
    }
}
