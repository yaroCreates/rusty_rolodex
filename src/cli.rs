use clap::{Parser, Subcommand};
use std::env;

use crate::domain::Contact;
use crate::store::mem::{AppError, ContactStore, FileStore, MemStore};
use crate::validation::{check_contact_exist, validate_email, validate_name, validate_phone_number, ValidationResponse};

#[derive(Parser)]
#[command(
    author,
    name = "rolodex",
    version = "1.0",
    about = "Contact CLI manager"
)]
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
        sort: Option<String>, // "name" or "email"
    },
    /// Delete a contact by name
    Delete {
        #[arg(long)]
        name: String,
    },
}

fn get_store() -> Box<dyn ContactStore> {
    match env::var("STORE_TYPE")
        .unwrap_or("file".to_string())
        .as_str()
    {
        "mem" => Box::new(MemStore::new()),
        _ => Box::new(FileStore::new("contacts.json")),
    }
}

pub fn run_command_cli() -> Result<(), AppError> {
    let cli = Cli::parse();
    let store = get_store();

    match cli.command {
        Commands::Add { name, phone, email } => {

            if !validate_name(&name) {
                return Err(AppError::Validation(ValidationResponse::check_name()));
            }

            if !validate_email(&email) {
                return Err(AppError::Validation(ValidationResponse::check_email()));
            }

            if !validate_phone_number(&phone) {
                return Err(AppError::Validation(ValidationResponse::check_phone_number()));
            }

            let mut contacts = store.load()?;
            let new_contact = Contact::new(&name, &phone, &email);

            if check_contact_exist(&new_contact, &contacts) {
                return Err(AppError::Validation(format!("Contact with name already exists")));
            }
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
