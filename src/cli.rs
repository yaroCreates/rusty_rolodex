use std::{env, fs};
use std::io::{self, Write};

use crate::domain::Contact;
use crate::store::mem::{AppError, ContactStore, FileStore, MemStore};
use crate::validation::{validate_email, validate_name, validate_phone};

pub fn run_cli() -> Result<(), AppError> {

    let store_type = env::var("STORE_TYPE").unwrap_or_else(|_| "file".to_string());

    let store: Box<dyn ContactStore> = match store_type.as_str() {
        "mem" => {
            println!("Using MemStore");
            Box::new(MemStore::new())
        }
        _ => {
            println!("Using FileStore");
            Box::new(FileStore::new("contacts.txt"))
        }
    };

    let mut contacts = store.load()?;

    // let mut store = load_contacts();

    loop {
        println!("\nWelcome to my Contact Manager\n");
        println!("1. Add a contact");
        println!("2. List all contacts");
        println!("3. Delete contact");
        println!("4. Exit");

        io::stdout().flush().unwrap();
        let menu_option = read_input();

        match menu_option.as_str() {
            "1" => add_contact(store.as_ref(), &mut contacts)?,
            "2" => view_contacts(&contacts),
            "3" => delete_contact(store.as_ref(), &mut contacts),
            "4" => {
                println!("Hope to see you back soon!");
                break;
            }
            _ => println!("Invalid selection, please choose from 1-4."),
        }
    }
    Ok(())
}

fn add_contact(storage: &dyn ContactStore, contacts: &mut Vec<Contact>) -> Result<(), AppError>{
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

    contacts.push(Contact::new(&name, &phone, &email));
    storage.save(contacts)?;
    println!("Contact added!");
    Ok(())
}

fn view_contacts(store: &Vec<Contact>) {
    if store.is_empty() {
        println!("No contact available")

    } else {
        for (i, c) in store.iter().enumerate() {
            println!(
                "{}. Name: {}, Phone: {}, Email: {}",
                i + 1,
                c.name,
                c.phone,
                c.email
            );
        }

    }

}

fn delete_contact(storage: &dyn ContactStore, contacts: &mut Vec<Contact>) {
    println!("\n--- Delete Contact ---");
    print!("Enter name to delete: ");
    io::stdout().flush().unwrap();
    let name = read_input();

    let length_before = contacts.len();
    contacts.retain(|c| c.name != name);

    if contacts.len() < length_before {
        println!("Contact deleted.");
        storage.save(contacts);
    } else {
        println!("Contact not found.");
    }
}

fn read_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}