mod cli;
mod domain;
mod validation;
mod store {
    pub mod mem;
}

use crate::cli::run_command_cli;

pub fn main() {
    if let Err(err) = run_command_cli() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
