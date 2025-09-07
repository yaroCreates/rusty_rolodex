use clap::{Parser, Subcommand};
use std::{env};
use std::io::{self, Write};

use crate::domain::Contact;
use crate::store::mem::{retry, AppError, ContactStore, FileStore, MemStore};
use crate::validation::{validate_email, validate_name, validate_phone};

#[derive(Parser)]
#[command(author, name = "rolodex", version = "1.0", about = "Contact CLI manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new contact
    Add {
        #[arg(long)]
        name: String,
        #[arg(long)]
        phone: String,
        #[arg(long)]
        email: String,
    },
    /// List contacts (optionally filter/sort)
    List {
        #[arg(long)]
        sort: Option<String>,   // "name" or "email"
    },
    /// Delete a contact by name
    Delete {
        #[arg(long)]
        name: String,
    },
}

fn get_store() -> Box<dyn ContactStore> {
    match env::var("STORE_TYPE").unwrap_or("file".to_string()).as_str() {
        "mem" => Box::new(MemStore::new()),
        _ => Box::new(FileStore::new("contacts.json")),
    }
}

pub fn run_cli() -> Result<(), AppError> {

    let store_type = env::var("STORE_TYPE").unwrap_or_else(|_| "file".to_string());

    let store: Box<dyn ContactStore> = match store_type.as_str() {
        "mem" => {
            println!("Using MemStore");
            Box::new(MemStore::new())
        }
        _ => {
            println!("Using FileStore");
            Box::new(FileStore::new("contacts.json"))
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
            "3" => delete_contact(store.as_ref(), &mut contacts)?,
            "4" => {
                println!("Hope to see you back soon!");
                break;
            }
            _ => println!("Invalid selection, please choose from 1-4."),
        }
    }
    Ok(())
}

pub fn run_command_cli() -> Result<(), AppError> {
    let cli = Cli::parse();
    let store = get_store();

    match cli.command {
        Commands::Add { name, phone, email } => {
            let mut contacts = store.load()?;
            contacts.push(Contact::new(&name, &phone, &email));
            store.save(&contacts)?;
            println!("✅ Added contact: {} ({})", name, email);
        }
        Commands::List { sort } => {
            let mut contacts = store.load()?;

            if let Some(sort_key) = sort {
                match sort_key.as_str() {
                    "name" => contacts.sort_by(|a, b| a.name.cmp(&b.name)),
                    "email" => contacts.sort_by(|a, b| a.email.cmp(&b.email)),
                    _ => println!("⚠️ Unsupported sort key: {}", sort_key),
                }
            }

            if contacts.is_empty() {
                println!("No contacts found.");
            } else {
                for c in contacts {
                    println!("📇 {} | {} | {}", c.name, c.phone, c.email);
                }
            }
        }
        Commands::Delete { name } => {
            let mut contacts = store.load()?;
            let len_before = contacts.len();
            contacts.retain(|c| c.name != name);

            if contacts.len() < len_before {
                store.save(&contacts)?;
                println!("🗑️ Removed contact: {}", name);
            } else {
                println!("⚠️ No contact found with name '{}'", name);
            }
        }
    }

    Ok(())
}


fn add_contact(storage: &dyn ContactStore, contacts: &mut Vec<Contact>) -> Result<(), AppError>{
    println!("\n--- Add Contact ---");

    let name: String = retry("Enter name: ", |s| {
        if !validate_name(&s) {
            Err(AppError::Parse("Name cannot be empty".into()))
        } else {
            Ok(s.to_string())
        }
    });


    let phone: String = retry("Enter phone: ", |s| {
        if validate_phone(&s) {
            Ok(s.to_string())
        } else {
            Err(AppError::Parse("Phone must contain only digits and be more than 10 digits".into()))
        }
    });

    let email: String = retry("Enter email: ", |s| {
        if validate_email(&s) {
            Ok(s.to_string())
        } else {
            Err(AppError::Parse("Invalid email format".into()))
        }
    });

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

fn delete_contact(storage: &dyn ContactStore, contacts: &mut Vec<Contact>) -> Result<(), AppError> {
    println!("\n--- Delete Contact ---");
    // print!("Enter name to delete: ");
    // io::stdout().flush().unwrap();
    // let name = read_input();

    let name: String = retry("Enter name to delete: ", |s| {
        if s.is_empty() {
            Err(AppError::Parse("Name cannot be empty".into()))
        } else {
            Ok(s.to_string())
        }
    });

    let length_before = contacts.len();
    contacts.retain(|c| c.name != name);

    if contacts.len() < length_before {
        storage.save(contacts)?;
        println!("Contact deleted.");
    } else {
        println!("Contact not found.");
    }
    Ok(())
}

fn read_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}