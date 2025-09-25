use rusty_rolodex::cli::run_command_cli;

pub fn main() {
    if let Err(err) = run_command_cli() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
