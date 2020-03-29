mod scanner;

use std::env;
use std::fs;
use std::io;
use std::io::prelude::*;

use scanner::{Position, Scanner};

pub struct Lox {
    had_error: bool,
}

impl Lox {
    fn new() -> Self {
        Lox { had_error: false }
    }

    fn run(&mut self, source: String) {
        let mut scanner = Scanner::new(self, source);
        let tokens = scanner.scan_tokens();

        for token in tokens {
            println!("{:?}", token);
        }
    }

    pub fn error(&mut self, position: Position, message: String) {
        self.report_error(position, "", message)
    }

    fn report_error(&mut self, position: Position, error_where: &str, message: String) {
        eprintln!("[{}] Error{}: {}", position, error_where, message);
        self.had_error = true;
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut lox = Lox::new();

    match args.len() {
        1 => run_prompt(&mut lox),
        2 => run_file(&mut lox, &args[1]),
        _ => {
            println!("Usage: loxrs script");
            std::process::exit(64);
        }
    }
}

fn run_file(lox: &mut Lox, path: &str) -> io::Result<()> {
    let source = fs::read_to_string(path).expect("Failed to read file");
    lox.run(source);
    if lox.had_error {
        println!("There was an error while running the file {}", path);
        std::process::exit(65);
    }
    Ok(())
}

fn run_prompt(lox: &mut Lox) -> io::Result<()> {
    loop {
        print!("> ");
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        lox.run(buffer);
        if lox.had_error {
            lox.had_error = false;
        }
    }
}
