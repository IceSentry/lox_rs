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
    cell::RefCell,
    fs,
    io::{self, prelude::*},
    rc::Rc,
};

use structopt::StructOpt;

use logger::{DefaultLogger, LoggerImpl};
use lox::{Lox, LoxError};

#[derive(Debug, StructOpt)]
#[structopt(name = "loxrs", about = "A rust implementation of a lox interpreter")]
struct Opt {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,

    /// Input file
    input: Option<String>,

    /// Print ast to <file>.ast.lox
    #[structopt(long)]
    ast: bool,
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();

    let mut logger = DefaultLogger::new(opt.debug, false);
    if let Some(input_path) = opt.input {
        run_file(logger, &input_path, opt.ast)
    } else {
        logger.is_repl = true;
        run_prompt(logger)
    }
}

fn run_file(logger: DefaultLogger, path: &str, print_ast: bool) -> io::Result<()> {
    let logger = Rc::new(RefCell::new(LoggerImpl::from(logger)));
    let mut lox = Lox::new(&logger, print_ast);
    let source = fs::read_to_string(path).expect("Failed to read file");
    let result = lox.run(&source);
    match result {
        Err(LoxError::Parser(_)) => std::process::exit(65),
        Err(LoxError::Panic(_)) => std::process::exit(70),
        _ => Ok(()),
    }
}

fn run_prompt(logger: DefaultLogger) -> io::Result<()> {
    let logger = Rc::new(RefCell::new(LoggerImpl::from(logger)));
    let mut lox = Lox::new(&logger, false);
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
