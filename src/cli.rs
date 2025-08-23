use std::io::{self, Write};

use crate::domain::Contact;
use crate::store::mem::ContactStore;
use crate::validation::{validate_email, validate_name, validate_phone};

pub fn run_cli() {
    let mut store = ContactStore::new();

    loop {
        println!("\nWelcome to my Contact Manager\n");
        println!("1. Add a contact");
        println!("2. List all contacts");
        println!("3. Delete contact");
        println!("4. Exit");

        io::stdout().flush().unwrap();
        let menu_option = read_input();

        match menu_option.as_str() {
            "1" => add_contact(&mut store),
            "2" => view_contacts(&store),
            "3" => delete_contact(&mut store),
            "4" => {
                println!("Hope to see you back soon!");
                break;
            }
            _ => println!("Invalid selection, please choose from 1-4."),
        }
    }
}

fn add_contact(store: &mut ContactStore) {
    println!("\n--- Add Contact ---");

    let name = loop {
        print!("Enter name: ");
        io::stdout().flush().unwrap();
        let input = read_input();
        if validate_name(&input) {
            break input;
        } else {
            println!("Name must be alphabetic and non-empty.");
        }
    };

    let phone = loop {
        print!("Enter phone: ");
        io::stdout().flush().unwrap();
        let input = read_input();
        if validate_phone(&input) {
            break input;
        } else {
            println!("Phone must be digits only and at least 10 digits.");
        }
    };

    let email = loop {
        print!("Enter email: ");
        io::stdout().flush().unwrap();
        let input = read_input();
        if validate_email(&input) {
            break input;
        } else {
            println!("Invalid email format.");
        }
    };

    store.add(Contact { name, phone, email });
}

fn view_contacts(store: &ContactStore) {
    for (i, c) in store.list().iter().enumerate() {
        println!(
            "{}. Name: {}, Phone: {}, Email: {}",
            i + 1,
            c.name,
            c.phone,
            c.email
        );
    }
}

fn delete_contact(store: &mut ContactStore) {
    println!("\n--- Delete Contact ---");
    print!("Enter name to delete: ");
    io::stdout().flush().unwrap();
    let name = read_input();

    if store.delete(&name) {
        println!("Contact deleted.");
    } else {
        println!("Contact not found.");
    }
}

fn read_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}
