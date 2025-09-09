use std::process::Command;

fn main() {
    let status = Command::new("cargo")
        .args(&[
            "run", "--", "list",
            "--sort", "name",
        ])
        .status()
        .expect("failed to run rolodex list");

    if !status.success() {
        eprintln!("rolodex list exited with error");
    }
}
