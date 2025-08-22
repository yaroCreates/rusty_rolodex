use std::io::{self, Write};

use regex::Regex;

struct Contact {
    name: String,
    phone: String,
    email: String,
}

fn main() {
    let mut contacts = Vec::new();
    contacts.push(Contact {
        name: "James Yaro".to_string(),
        phone: "08122121474".to_string(),
        email: "onuhjamesyaro@gmail.com".to_string(),
    });

    loop {
        println!("\n");
        println!("Welcome to my Contact Manager");
        println!("\n");
        println!("1. Add a contact");
        println!("2. List all contacts");
        println!("3. Delete contact");
        println!("4. Exit");
    
        io::stdout().flush().unwrap();
        let menu_option = read_input();

        match menu_option.as_str() {
            "1" => {
                add_contact(&mut contacts);
            }
            "2" => view_contacts(&mut contacts),
            "3" => {
                delete_contact(&mut contacts);
            }
            "4" => {
                println!("Hope to see you back soon!");
                break;
            }
            _ => println!("Invalid selection, please choose from 1-3."),
        }
    
        

    }

}

fn add_contact(contacts: &mut Vec<Contact>) {
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

    contacts.push(Contact {
        name,
        phone,
        email,
    });
}

fn view_contacts(contacts: &mut Vec<Contact>) {
    for (i, c) in contacts.iter().enumerate() {
        println!(
            "{}. Name: {}, Phone: {}, Email: {}",
            i + 1,
            c.name,
            c.phone,
            c.email
        )
    }
}

fn read_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

// Validation

fn validate_name(name: &str) -> bool {
    !name.trim().is_empty() && name.chars().all(|c| c.is_alphabetic() || c.is_whitespace())
}

fn validate_phone(phone: &str) -> bool {
    let re = Regex::new(r"^\d{10,}$").unwrap();
    re.is_match(phone)
}

fn validate_email(email: &str) -> bool {
    let re = Regex::new(r"^[\w\.-]+@[\w\.-]+\.\w+$").unwrap();
    re.is_match(email)
}

fn delete_contact(contacts: &mut Vec<Contact>) {
    println!("\n--- Delete Contact ---");
    print!("Enter name to delete: ");
    io::stdout().flush().unwrap();
    let name = read_input();

    let initial_len = contacts.len();
    contacts.retain(|c| c.name.to_lowercase() != name.to_lowercase());

    if contacts.len() < initial_len {
        println!("Contact deleted.");
    } else {
        println!("Contact not found.");
    }
}
