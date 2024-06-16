use std::{fs, io};

use ast_printer::AstPrinter;
use expr::test_ast_print;
use parser::Parser;
use scanner::Scanner;

mod ast_printer;
mod expr;
mod parser;
mod scanner;

struct RuntimeError;

pub fn run_file(path: &str) {
    let bytes = fs::read(path).expect("file to be readable");
    // run code
}

pub fn run_prompt() {
    test_ast_print();

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
    // scan tokens
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();

    let mut parser = Parser::new(tokens);
    let expression = parser.parse();

    let printer = AstPrinter {};
    printer.print(&expression);
}

// calling code will throw error
pub fn print_error(line: u64, location: String, message: String) {
    eprintln!("[line {line}] Error {location}: {message}");
}
