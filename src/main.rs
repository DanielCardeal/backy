mod commands;
mod config;
mod error;

use commands::Command;
use error::BackyError;
use std::{env, process::exit};

fn run_app() -> Result<(), BackyError> {
    let args: Vec<String> = env::args().collect();
    let config = config::load()?;
    let command = Command::from_args(&args)?;
    command.execute(config)?;
    Ok(())
}

fn main() {
    if let Err(err) = run_app() {
        eprintln!("{}", err);
        exit(1);
    }
}
