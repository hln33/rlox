use std::{fs, io, process};

use interpreter::Interpreter;
use parser::Parser;
use resolver::Resolver;
use scanner::Scanner;
use syntax::{token::Token, value::Value};
pub use utils::logger::Logger;

mod environment;
mod impls;
mod interpreter;
mod parser;
mod resolver;
mod scanner;
mod syntax;
mod utils;

static mut HAD_RUNTIME_ERROR: bool = false;

#[derive(Debug)]
enum Exception {
    RuntimeError(RuntimeError),
    Return(Value),
}

impl Exception {
    fn runtime_error<T>(token: Token, message: String) -> Result<T, Exception> {
        Err(Exception::RuntimeError(RuntimeError { token, message }))
    }
}

#[derive(Debug)]
struct RuntimeError {
    token: Token,
    message: String,
}

impl RuntimeError {
    fn error(&self) {
        println!("{}", self.message);
        println!("[line {}]", self.token.line);

        unsafe { HAD_RUNTIME_ERROR = true }
    }
}

pub fn runtime_error() -> bool {
    unsafe { HAD_RUNTIME_ERROR }
}

fn check_runtime_error() {
    unsafe {
        if HAD_RUNTIME_ERROR {
            process::exit(70)
        }
    }
}

pub fn run_file(path: &str, logger: Option<Box<dyn Logger>>) {
    // let _bytes = fs::read(path).expect("file to be readable");

    let mut interpreter = Interpreter::new(logger);
    let contents = fs::read_to_string(path).expect("file to be readable");
    run(contents, &mut interpreter);

    unsafe {
        if HAD_RUNTIME_ERROR {
            process::exit(70)
        }
    }
}

pub fn run_prompt() {
    let mut interpreter = Interpreter::new(None);

    loop {
        println!("> ");

        let mut user_input = String::new();
        io::stdin()
            .read_line(&mut user_input)
            .expect("valid user input");

        let user_input = user_input.trim();
        if user_input == "exit" {
            break;
        }

        run(user_input.to_string(), &mut interpreter);
    }
}

fn run(source: String, interpreter: &mut Interpreter) {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();

    let mut parser = Parser::new(tokens);
    let statements = parser.parse();

    check_runtime_error();

    let mut resolver = Resolver::new(interpreter);
    resolver.resolve_block(&statements);

    check_runtime_error();

    interpreter.interpret(statements);
}

// calling code will throw error
pub fn print_error(line: usize, location: String, message: &str) {
    eprintln!("[line {line}] Error {location}: {message}");
}
