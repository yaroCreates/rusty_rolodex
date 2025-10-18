use assert_cmd::Command;

fn run_rolodex(args: &[&str], data_dir: &assert_fs::TempDir) -> String {
    println!("Using ROLODEX_DATA_DIR: {}", data_dir.path().display());

    unsafe { std::env::set_var("ROLODEX_DATA_DIR", data_dir.path()) };

    let mut cmd = Command::cargo_bin("rolodex").unwrap();
    let output = cmd
        .args(args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8_lossy(&output).to_string()
}

#[test]
fn cli_add_and_list_contact() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Add a contact
    let add_out = run_rolodex(
        &[
            "add",
            "--name",
            "Alice",
            "--phone",
            "123345534343434",
            "--email",
            "alice@work.com",
            "--tags",
            "work",
        ],
        &temp,
    );
    assert!(add_out.contains("Added contact"));

    // List and assert output contains contact name
    let list_out = run_rolodex(&["list"], &temp);
    assert!(list_out.contains("Alice"));
    assert!(list_out.contains("alice@work.com"));

    temp.close().unwrap();
}

// #[test]
// fn cli_delete_contact() {
//     let temp = assert_fs::TempDir::new().unwrap();

//     // Add > Delete > List
//     run_rolodex(
//         &[
//             "add",
//             "--name",
//             "Bob",
//             "--phone",
//             "4568989808333",
//             "--email",
//             "bob@home.com",
//             "--tags",
//             "work",
//         ],
//         &temp,
//     );
//     let del_out = run_rolodex(&["delete", "--name", "Bob"], &temp);
//     assert!(del_out.contains("Removed contact"));

//     let list_out = run_rolodex(&["list"], &temp);
//     assert!(!list_out.contains("Bob"));

//     temp.close().unwrap();
// }
