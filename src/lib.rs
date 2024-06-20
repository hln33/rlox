use std::{fs, io, process};

use interpreter::Interpreter;
use parser::Parser;
use scanner::{Scanner, Token};

mod ast_printer;
mod expr;
mod interpreter;
mod parser;
mod scanner;
mod stmt;

static mut HAD_RUNTIME_ERROR: bool = false;

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
    let bytes = fs::read(path).expect("file to be readable");
    // run code

    unsafe {
        if HAD_RUNTIME_ERROR {
            process::exit(70)
        }
    }
}

pub fn run_prompt() {
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
        run(user_input.to_string());
    }
}

fn run(source: String) {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();

    let mut parser = Parser::new(tokens);
    let statements = parser.parse();

    let interpreter = Interpreter;
    interpreter.interpret(statements);
}

// calling code will throw error
pub fn print_error(line: u64, location: String, message: String) {
    eprintln!("[line {line}] Error {location}: {message}");
}
