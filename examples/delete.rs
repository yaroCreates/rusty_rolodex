use std::process::Command;

fn main() {
    let status = Command::new("cargo")
        .args(&[
            "run", "--", "delete",
            "--name", "Alice",
        ])
        .status()
        .expect("failed to run rolodex delete");

    if !status.success() {
        eprintln!("rolodex delete exited with error");
    }
}
