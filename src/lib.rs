use std::{fs, io, process};

use interpreter::Interpreter;
use parser::Parser;
use resolver::Resolver;
use scanner::{Scanner, Token};
use value::Value;

// mod ast_printer;
mod Function;
mod environment;
mod expr;
mod interpreter;
mod logger;
mod parser;
mod resolver;
mod scanner;
mod stmt;
mod value;

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

pub fn run_file(path: &str) {
    // let _bytes = fs::read(path).expect("file to be readable");

    let mut interpreter = Interpreter::new();
    let contents = fs::read_to_string(path).expect("file to be readable");
    run(contents, &mut interpreter);

    unsafe {
        if HAD_RUNTIME_ERROR {
            process::exit(70)
        }
    }
}

pub fn run_prompt() {
    let mut interpreter = Interpreter::new();

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
        // todo run line of code
        run(user_input.to_string(), &mut interpreter);
    }
}

fn run(source: String, interpreter: &mut Interpreter) {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();

    let mut parser = Parser::new(tokens);
    let statements = parser.parse();

    unsafe {
        if HAD_RUNTIME_ERROR {
            process::exit(70)
        }
    }

    let mut resolver = Resolver::new(interpreter);
    resolver.resolve_block(&statements);

    interpreter.interpret(statements);
}

// calling code will throw error
pub fn print_error(line: usize, location: String, message: &str) {
    eprintln!("[line {line}] Error {location}: {message}");
}
