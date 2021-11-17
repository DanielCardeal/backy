mod commands;

use commands::Command;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = Command::from_args(&args);
    command.execute()
}
