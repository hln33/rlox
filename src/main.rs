use std::{cmp::Ordering, env, process};

use rlox::{run_file, run_prompt};

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("{:?}", args);

    match args.len().cmp(&2) {
        Ordering::Greater => {
            println!("Usage: rlox [script]");
            process::exit(64);
        }
        Ordering::Equal => {
            run_file(&args[1]);
        }
        _ => {
            run_prompt();
        }
    }
}
