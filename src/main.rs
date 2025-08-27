
mod cli;
mod domain;
mod validation;
mod store {
    pub mod mem;
}

use crate::cli::run_cli;

fn main() {
    run_cli();
}
