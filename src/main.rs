mod environment;
mod expr;
mod interpreter;
mod literal;
mod logger;
mod lox;
mod parser;
mod scanner;
mod stmt;
mod token;

#[cfg(test)]
mod tests;

use std::env;
use std::fs;
use std::io;
use std::io::prelude::*;

use logger::DefaultLogger;
use lox::{Lox, LoxError};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let debug = false; //TODO use structop for flag

    match args.len() {
        1 => {
            let mut logger = DefaultLogger::new(debug, true);
            let mut lox = Lox::new(&mut logger);
            run_prompt(&mut lox)
        }
        2 => {
            let mut logger = DefaultLogger::new(debug, false);
            let mut lox = Lox::new(&mut logger);
            run_file(&mut lox, &args[1])
        }
        _ => {
            println!("Usage: loxrs script");
            std::process::exit(64);
        }
    }
}

fn run_file(lox: &mut Lox, path: &str) -> io::Result<()> {
    let source = fs::read_to_string(path).expect("Failed to read file");
    let result = lox.run(&source);
    match result {
        Err(LoxError::Parser(_)) => std::process::exit(65),
        // Err(LoxError::Runtime(_)) => std::process::exit(70),
        Ok(_) => Ok(()),
    }
}

fn run_prompt(lox: &mut Lox) -> io::Result<()> {
    println!("lox prompt: ");
    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;

        if buffer.trim() == "exit" {
            std::process::exit(0);
        }

        lox.run(&buffer).ok();
    }
}
