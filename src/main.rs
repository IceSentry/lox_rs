mod environment;
mod expr;
mod function;
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

use std::{
    fs,
    io::{self, prelude::*},
};

use structopt::StructOpt;

use logger::{DefaultLogger, Logger};
use lox::{Lox, LoxError};

#[derive(Debug, StructOpt)]
#[structopt(name = "loxrs", about = "A rust implementation of a lox interpreter")]
struct Opt {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,

    /// Input file
    input: Option<String>,
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();

    let mut logger = DefaultLogger::new(opt.debug, false);
    if let Some(input_path) = opt.input {
        run_file(&mut logger, &input_path)
    } else {
        logger.is_repl = true;
        run_prompt(&mut logger)
    }
}

fn run_file(logger: &mut dyn Logger, path: &str) -> io::Result<()> {
    let mut lox = Lox::new(logger);
    let source = fs::read_to_string(path).expect("Failed to read file");
    let result = lox.run(&source);
    match result {
        Err(LoxError::Parser(_)) => std::process::exit(65),
        // Err(LoxError::Runtime(_)) => std::process::exit(70),
        Ok(_) => Ok(()),
    }
}

fn run_prompt(logger: &mut dyn Logger) -> io::Result<()> {
    let mut lox = Lox::new(logger);
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
