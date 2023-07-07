use std::io::Write;

mod expr;
mod parser;
mod scanner;
mod token;

use crate::expr::{Binary, Expr, Grouping, Literal};
use crate::token::{Token, TokenType};

static mut HAD_ERROR: bool = false;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() >= 2 {
        run_file(args[1].clone());
    } else {
        run_prompt();
    }
}

fn run_file(file_path: String) {
    println!("Running file: {}", file_path);
    let source =
        std::fs::read_to_string(&file_path).expect("Something went wrong reading the file");
    run(source);

    if unsafe { HAD_ERROR } {
        std::process::exit(65);
    }
}

fn run_prompt() {
    println!("Welcome to the Lox REPL!");
    println!("Press q to quit.");
    loop {
        let mut input = String::new();
        print!("> ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input.to_lowercase() == "q" {
            break;
        }
        run(input.to_string());
        unsafe {
            HAD_ERROR = false;
        }
    }
    println!("Bye!");
}

fn run(input: String) {
    let mut scanner = scanner::Scanner::new(input);
    let tokens = scanner.scan_tokens();
    let mut parser = crate::parser::Parser::new(tokens);
    if let Ok(expr) = parser.parse() {
        println!("{}", crate::expr::print(expr))
    }
}

fn error(line: usize, message: &str) {
    report(line, "", message);
}

fn report(line: usize, location: &str, message: &str) {
    eprintln!("Error: [line {}] Error {}: {}", line, location, message);
}
