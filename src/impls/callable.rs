use crate::{
    interpreter::Interpreter,
    syntax::{token::Token, value::Value},
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
