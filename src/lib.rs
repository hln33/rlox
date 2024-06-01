use std::{fs, io};

mod scanner;

struct RuntimeError;

pub fn run_file(path: &str) {
    let bytes = fs::read(path).expect("file to be readable");
    // run code
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
    }
}

// calling code will throw error
fn print_error(line: u64, location: String, message: String) {
    eprintln!("[line {line}] Error {location}: {message}");
}

fn run(source: String) -> Result<(), RuntimeError> {
    // scan tokens

    Ok(())
}
