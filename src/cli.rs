use clap::{Parser, Subcommand};
use std::env;

use crate::domain::{Contact, Contacts};
use crate::store::mem::{AppError, ContactStore, FileStore, MemStore};
use crate::validation::{
    ValidationResponse, check_contact_exist, validate_email, validate_name, validate_phone_number,
};

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
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
    },
    /// List contacts (optionally filter/sort)
    List {
        #[arg(long)]
        sort: Option<String>, // "name" or "email"
        #[arg(long)]
        tag: Option<String>,
        #[arg(long)]
        domain: Option<String>,
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
        Commands::Add {
            name,
            phone,
            email,
            tags,
        } => {
            if !validate_name(&name) {
                return Err(AppError::Validation(ValidationResponse::check_name()));
            }

            if !validate_email(&email) {
                return Err(AppError::Validation(ValidationResponse::check_email()));
            }

            if !validate_phone_number(&phone) {
                return Err(AppError::Validation(
                    ValidationResponse::check_phone_number(),
                ));
            }

            let mut contacts = store.load()?;
            let new_contact = Contact::new(&name, &phone, &email, tags.clone());

            if check_contact_exist(&new_contact, &contacts) {
                return Err(AppError::Validation(
                    "Contact with name already exists".to_string(),
                ));
            }
            contacts.push(Contact::new(&name, &phone, &email, tags));
            store.save(&contacts)?;
            println!("✅ Added contact: {} ({})", name, email);
        }
        Commands::List { sort, tag, domain } => {
            let contacts_vec = store.load()?;
            let contacts = Contacts::new(contacts_vec);

            //Chain filters using iterator
            let mut filtered_contacts: Vec<&Contact> = contacts
                .as_slice()
                .iter()
                .filter(|c| tag.as_ref().map_or(true, |t| c.has_tag(t)))
                .filter(|c| domain.as_ref().map_or(true, |d| c.has_domain(d)))
                .collect();

            if let Some(sort_key) = sort {
                match sort_key.as_str() {
                    "name" => filtered_contacts.sort_by(|a, b| a.name.cmp(&b.name)),
                    "email" => filtered_contacts.sort_by(|a, b| a.email.cmp(&b.email)),
                    _ => println!("⚠️ Unsupported sort key: {}", sort_key),
                }
            }

            if filtered_contacts.is_empty() {
                println!("No contacts found.");
            } else {
                for c in filtered_contacts {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_contacts() -> Contacts {
        Contacts::new(vec![
            Contact::new("Alice", "123", "alice@work.com", vec!["work".into()]),
            Contact::new("Bob", "456", "bob@personal.com", vec!["personal".into()]),
            Contact::new("Carol", "789", "carol@work.com", vec!["work".into()]),
        ])
    }

    #[test]
    fn test_filter_by_tag() {
        let contacts = sample_contacts();
        let work: Vec<_> = contacts
            .as_slice()
            .into_iter()
            .filter(|c| c.has_tag("work"))
            .collect();
        assert_eq!(work.len(), 2);
    }

    #[test]
    fn test_filter_by_domain() {
        let contacts = sample_contacts();
        let work_mails: Vec<_> = contacts
            .as_slice()
            .into_iter()
            .filter(|c| c.has_domain("work.com"))
            .collect();
        assert_eq!(work_mails.len(), 2);
    }

    #[test]
    fn test_chainable_filters() {
        let contacts = sample_contacts();
        let results: Vec<_> = contacts
            .as_slice()
            .into_iter()
            .filter(|c| c.has_tag("work"))
            .filter(|c| c.has_domain("work.com"))
            .take(1)
            .collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Alice");
    }
}

//Borrow check test
#[test]
fn test_as_slice() {
    let contacts = Contacts::new(vec![
        Contact::new("Alice", "123", "alice@work.com", vec!["work".into()]),
        Contact::new("Bob", "456", "bob@home.com", vec![]),
    ]);

    let slice = contacts.as_slice();
    assert_eq!(slice.len(), 2);
    assert_eq!(slice[0].name, "Alice");
}
