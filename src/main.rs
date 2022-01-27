mod commands;
mod config;
mod error;

use error::BackyResult;
use std::{env, process::exit};

fn run_app() -> BackyResult {
    let args: Vec<String> = env::args().collect();
    let config = config::load()?;
    let command = commands::from_args(&args)?;
    command.execute(config)?;
    Ok(())
}

fn main() {
    if let Err(err) = run_app() {
        err.display();
        exit(1);
    }
}
