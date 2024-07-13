use crate::{
    class::ClassInstanceRef,
    environment::{EnvRef, Environment},
    interpreter::Interpreter,
    scanner::Token,
    stmt::Stmt,
    value::Value,
    Exception,
};

pub trait Callable {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, Exception>;
    fn check_arity(&self, args_len: usize, current_token: &Token) -> Result<(), Exception> {
        if args_len > self.arity() {
            return Exception::runtime_error(
                current_token.clone(),
                format!("Expected {} arguments but got {}.", self.arity(), args_len),
            );
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct NativeFunction {
    pub arity: usize,
    pub callable: fn(&mut Interpreter, Vec<Value>) -> Value,
}

impl Callable for NativeFunction {
    fn arity(&self) -> usize {
        self.arity
    }

    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, Exception> {
        Ok((self.callable)(interpreter, args))
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    declaration: Stmt,
    closure: EnvRef,
    is_initializer: bool,
}

impl Function {
    pub fn new(declaration: Stmt, closure: EnvRef, is_initializer: bool) -> Function {
        match &declaration {
            Stmt::Function { .. } => Function {
                declaration,
                closure,
                is_initializer,
            },
            _ => panic!("Function was not initialized with a function declaration!"),
        }
    }

    pub fn bind(&self, instance: ClassInstanceRef) -> Function {
        let environment = Environment::new_local(&self.closure);
        environment
            .borrow_mut()
            .define(String::from("this"), Value::ClassInstance(instance));

        Function::new(self.declaration.clone(), environment, self.is_initializer)
    }
}

impl Callable for Function {
    fn arity(&self) -> usize {
        if let Stmt::Function { params, .. } = &self.declaration {
            return params.len();
        }
        panic!("Function was not initialized with a function declaration!");
    }

    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, Exception> {
        let environment = Environment::new_local(&self.closure);

        if let Stmt::Function { params, body, .. } = &self.declaration {
            for (i, param) in params.iter().enumerate() {
                environment
                    .borrow_mut()
                    .define(param.lexeme.clone(), args.get(i).unwrap().clone())
            }

            if let Err(exception) = interpreter.execute_block(body, environment) {
                return match exception {
                    Exception::RuntimeError(e) => Err(Exception::RuntimeError(e)),
                    Exception::Return(value) => match self.is_initializer {
                        true => self.closure.borrow().get_at(0, "this"),
                        false => Ok(value),
                    },
                };
            }
        }

        if self.is_initializer {
            return self.closure.borrow().get_at(0, "this");
        }
        Ok(Value::Nil)
    }
}
