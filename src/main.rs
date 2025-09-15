mod cli;
mod domain;
mod validation;
mod store {
    pub mod mem;
}

use crate::cli::run_command_cli;

pub fn main() {
    let _ = run_command_cli();
}
