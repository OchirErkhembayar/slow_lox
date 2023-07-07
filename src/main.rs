use std::io::Write;

mod token;
mod scanner;
mod expr;
mod parser;

use crate::expr::{Binary, Expr, Literal, Grouping};
use crate::token::{Token, TokenType};

static mut HAD_ERROR: bool = false;

fn main() {
    let token_number = Token::new(TokenType::NUMBER, String::from("123"), 1);
    let literal1 = Box::new(
        Expr::Literal(
            Literal {
                value: &token_number,
            },
        )
    );
    let token_number_2 = Token::new(TokenType::NUMBER, String::from("456"), 1);
    let literal2 = Box::new(
        Expr::Literal(
            Literal {
                value: &token_number_2,
            },
        )
    );
    let token_number_3 = Token::new(TokenType::NUMBER, String::from("789"), 1);
    let literal3 = Box::new(
        Expr::Literal(
            Literal {
                value: &token_number_3,
            },
        )
    );
    let token_star = Token::new(TokenType::STAR, String::from("*"), 1);
    let expr2 = Expr::Grouping(
        Grouping {
            expression: Box::new(Expr::Binary(
                Binary {
                    left: literal2,
                    operator: &token_star,
                    right: literal3,
                },
            )),
        },
    );
    let token_star = Token::new(TokenType::STAR, String::from("*"), 1);
    let expr = Expr::Binary(
        Binary {
            left: literal1,
            operator: &token_star,
            right: Box::new(expr2),
        }
    );
    println!("{}", crate::expr::print(expr));

    let args = std::env::args().collect::<Vec<String>>();
    if args.len() >= 2 {
        run_file(args[1].clone());
    } else {
        run_prompt();
    }
}

fn run_file(file_path: String) {
    println!("Running file: {}", file_path);
    let source = std::fs::read_to_string(&file_path).expect("Something went wrong reading the file");
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
    for token in tokens {
        println!("{}", token.to_string());
    }
}

fn error(line: usize, message: &str) {
    report(line, "", message);
}

fn report(line: usize, location: &str, message: &str) {
    eprintln!("Error: [line {}] Error {}: {}", line, location, message);
}
