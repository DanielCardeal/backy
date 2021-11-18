mod commands;
mod error;

use commands::Command;
use std::{env, process::exit};

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = Command::from_args(&args).unwrap_or_else(|err| {
        eprintln!("{}", err);
        exit(1);
    });
    command.execute();
}
