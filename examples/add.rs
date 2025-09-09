use std::process::Command;

fn main() {
    let status = Command::new("cargo")
    .args(&[
        "run", "--", "add",
        "--name", "Alice",
        "--phone", "1234567890",
        "--email", "alice@example.com"
    ])
    .status()
    .expect("failed to run rolodex");

if !status.success() {
    eprintln!("rolodex exited with error");
}

}