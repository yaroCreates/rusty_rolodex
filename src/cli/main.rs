use rusty_rolodex::cli::run_command_cli;

#[tokio::main]
pub async fn main() {
    if let Err(err) = run_command_cli().await {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
