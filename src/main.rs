use std::io::Write;

static mut HAD_ERROR: bool = false;

fn main() {

    let args = std::env::args().collect::<Vec<String>>();
    if args.len() >= 2 {
        run_file(&args[1]);
    } else {
        run_prompt();
    }
}

fn run_file(source: &String) {
    run(&source);

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
        run(input);
        unsafe {
            HAD_ERROR = false;
        }
    }
    println!("Bye!");
}

fn run(input: &str) {
    println!("{}", input);
}

fn error(line: usize, message: &str) {
    report(line, "", message);
}

fn report(line: usize, location: &str, message: &str) {
    eprintln!("Error: [line {}] Error {}: {}", line, location, message);
}
