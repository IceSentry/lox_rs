mod expr;
mod interpreter;
mod lox;
mod parser;
mod scanner;

use std::env;
use std::fs;
use std::io;
use std::io::prelude::*;

use lox::Lox;

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
    lox.run(&source);
    if lox.had_error {
        println!("There was an error while running the file {}", path);
        std::process::exit(65);
    }
    if lox.had_runtime_error {
        std::process::exit(70);
    }
    Ok(())
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

        lox.run(&buffer);
        if lox.had_error {
            lox.had_error = false;
        }
    }
}
