use chrono::Utc;
use clap::{Parser, Subcommand};
use std::env;

use crate::domain::{Contact, Contacts, export_csv, import_csv};
use crate::store::mem::{AppError, FileStore, MemStore, MergePolicy};
use crate::traits::ContactStore;
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
        // #[arg(long)]
        // created_at: Option<String>,
        // #[arg(long)]
        // updated_at: Option<String>
    },
    /// Delete a contact by name
    Delete {
        #[arg(long)]
        name: String,
        #[arg(long)]
        phone: Option<String>,
    },
    Update {
        #[arg(long)]
        name: String,
        #[arg(long)]
        phone: String,
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
        //For update
        #[arg(long)]
        new_name: Option<String>,
        #[arg(long)]
        new_phone: Option<String>,
        #[arg(long)]
        new_email: Option<String>,
    },
    ExportCsv {
        #[arg(long, default_value = "contacts.csv")]
        path: String,
    },
    ImportCsv {
        #[arg(long, default_value = "contacts.csv")]
        path: String,
    },
    Search {
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        domain: Option<String>,
        #[arg(long)]
        fuzzy: Option<String>,
    },
    Sync {
        #[arg(long)]
        file: String,
        #[arg(long, default_value = "keep")]
        policy: String,
    },
}

fn get_store() -> Box<dyn ContactStore> {
    match env::var("STORE_TYPE")
        .unwrap_or("file".to_string())
        .as_str()
    {
        "mem" => Box::new(MemStore::new()),
        _ => Box::new(FileStore),
    }
}

pub fn run_command_cli() -> Result<(), AppError> {
    let cli = Cli::parse();
    let store = get_store();

    let mut contacts = Contacts::new(store.load()?);


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

            // let mut contacts = store.load()?;
            let new_contact =
                Contact::new(&name, &phone, &email, tags.clone(), Utc::now(), Utc::now());

            
            contacts.add(new_contact)?;
            store.save(&contacts.items)?;
            println!("✅ Added contact: {} ({})", name, email);
        }
        Commands::List { sort, tag, domain } => {
            let contacts_vec = store.load()?;
            let contacts = Contacts::new(contacts_vec);

            //Chain filters using iterator
            let mut filtered_contacts: Vec<&Contact> = contacts
                .iter()
                .filter(|c| tag.as_ref().is_none_or(|t| c.has_tag(t)))
                .filter(|c| domain.as_ref().is_none_or(|d| c.has_domain(d)))
                .collect();

            if let Some(sort_key) = sort {
                match sort_key.as_str() {
                    "name" => filtered_contacts.sort_by(|a, b| a.name.cmp(&b.name)),
                    "email" => filtered_contacts.sort_by(|a, b| a.email.cmp(&b.email)),
                    "created_at" => {
                        filtered_contacts.sort_by(|a, b| a.created_at.cmp(&b.created_at))
                    }
                    "updated_at" => {
                        filtered_contacts.sort_by(|a, b| a.updated_at.cmp(&b.updated_at))
                    }
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
        Commands::Delete { name, phone } => {
            contacts.delete(name, phone)?;
            store.save(&contacts.items)?;


        }
        Commands::Update {
            name,
            phone,
            tags,
            new_name,
            new_phone,
            new_email,
        } => {
            if !validate_name(&name) {
                return Err(AppError::Validation(ValidationResponse::check_name()));
            }

            if !validate_phone_number(&phone) {
                return Err(AppError::Validation(
                    ValidationResponse::check_phone_number(),
                ));
            }

            //validation for new data -> Update
            let new_name = new_name.unwrap_or_default();
            let new_phone = new_phone.unwrap_or_default();
            let new_email = new_email.unwrap_or_default();

            let mut contacts = store.load()?;

            if let Some(contact) = contacts
                .iter_mut()
                .find(|c| c.name == name && c.phone == phone)
            {
                if !new_name.is_empty() {
                    if !validate_name(&new_name) {
                        return Err(AppError::Validation("New name is Invalid".to_string()));
                    }
                    contact.name = new_name.clone();
                }

                if !new_phone.is_empty() {
                    if !validate_phone_number(&new_phone) {
                        return Err(AppError::Validation(
                            "New phone number is Invalid".to_string(),
                        ));
                    }
                    contact.phone = new_phone.clone();
                }

                if !new_email.is_empty() {
                    if !validate_email(&new_email) {
                        return Err(AppError::Validation("New email is Invalid".to_string()));
                    }
                    contact.email = new_email.clone();
                }
                contact.tags = tags;
                contact.updated_at = Utc::now();

                store.save(&contacts)?;

                println!(
                    "✅ Contact updated: {} - {} ->  {} - {} - {}",
                    name, phone, new_name, new_phone, new_email
                );
            } else {
                return Err(AppError::Parse(format!(
                    "Contact with name '{}' and phone '{}' not found",
                    name, phone
                )));
            }
        }
        Commands::ExportCsv { path } => {
            let contacts = store.load()?;
            export_csv(&path, &contacts)?;
            println!("✅ Exported {} contacts to {}", contacts.len(), path);
        }
        Commands::ImportCsv { path } => {
            let mut contacts = store.load()?;
            let imported = import_csv(&path)?;
            contacts.extend(imported);
            store.save(&contacts)?;
            println!("✅ Imported contacts from {}", path);
        }
        Commands::Search {
            name,
            domain,
            fuzzy,
        } => {
            let name = name.unwrap_or_default();
            let domain = domain.unwrap_or_default();
            let fuzzy = fuzzy.unwrap_or_default();

            let _details = store.search(name, domain, fuzzy)?;
        }
        Commands::Sync { file, policy } => {
            let merge_policy = match policy.as_str() {
                "keep" => MergePolicy::Keep,
                "overwrite" => MergePolicy::Overwrite,
                "duplicate" => MergePolicy::Duplicate,
                _ => {
                    eprintln!(
                        "❌ Invalid policy '{}'. Use: keep | overwrite | duplicate",
                        policy
                    );
                    return Ok(());
                }
            };
            store.merge_from_file(&file, merge_policy)?;
            println!("✅ Sync complete using policy: {}", policy);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use std::time::Instant;

    use crate::store::mem::ContactsIndex;

    use super::*;

    fn sample_contacts() -> Contacts {
        Contacts::new(vec![
            Contact::new(
                "Alice",
                "123",
                "alice@work.com",
                vec!["work".into()],
                Utc::now(),
                Utc::now(),
            ),
            Contact::new(
                "Bob",
                "456",
                "bob@personal.com",
                vec!["personal".into()],
                Utc::now(),
                Utc::now(),
            ),
            Contact::new(
                "Carol",
                "789",
                "carol@work.com",
                vec!["work".into()],
                Utc::now(),
                Utc::now(),
            ),
        ])
    }

    fn sample_contacts2() -> Vec<Contact> {
        vec![
            Contact::new(
                "Alice",
                "123",
                "alice@work.com",
                vec!["work".into()],
                Utc::now(),
                Utc::now(),
            ),
            Contact::new(
                "Alicia",
                "123",
                "alicia@work.com",
                vec!["work".into()],
                Utc::now(),
                Utc::now(),
            ),
            Contact::new(
                "Bob",
                "456",
                "bob@personal.com",
                vec!["personal".into()],
                Utc::now(),
                Utc::now(),
            ),
            Contact::new(
                "Carol",
                "789",
                "carol@work.com",
                vec!["work".into()],
                Utc::now(),
                Utc::now(),
            ),
        ]
    }

    #[test]
    fn test_filter_by_tag() {
        let contacts = sample_contacts();
        let work: Vec<_> = contacts.iter().filter(|c| c.has_tag("work")).collect();
        assert_eq!(work.len(), 2);
    }

    #[test]
    fn test_filter_by_domain() {
        let contacts = sample_contacts();
        let work_mails: Vec<_> = contacts
            .iter()
            .filter(|c| c.has_domain("work.com"))
            .collect();
        assert_eq!(work_mails.len(), 2);
    }

    #[test]
    fn test_chainable_filters() {
        let contacts = sample_contacts();
        let results: Vec<_> = contacts
            .iter()
            .filter(|c| c.has_tag("work"))
            .filter(|c| c.has_domain("work.com"))
            .take(1)
            .collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Alice");
    }

    #[test]
    fn test_index_and_lookup() {
        let contacts = sample_contacts2();
        let index = ContactsIndex::build(&contacts);

        let position = index.lookup_name("Alice");
        assert_eq!(position.len(), 1);
        assert_eq!(contacts[position[0]].email, "alice@work.com");

        let domain_results = index.lookup_domain("work.com");
        //There are two contacts with "work.com" domain: Carol and Alice
        assert_eq!(domain_results.len(), 3);
    }

    // fuzzy search
    #[test]
    fn test_exact_match_name() {
        let contacts = sample_contacts2();
        let index = ContactsIndex::build(&contacts);
        let results = index.fuzzy_search("Alice", &contacts, 1);
        println!("Results {:?}", results);
        assert_eq!(results, vec![0]);
    }

    // benchmarking
    // 1. Concurrent fuzzy search
    #[test]
    fn benchmark_fuzzy_search_concurrent() {
        // Generate 10,000 fake contacts
        let contacts: Vec<_> = (0..10_000)
            .map(|i| Contact {
                name: format!("Person{}", i),
                email: format!("person{}@mail.com", i),
                phone: format!("232323323211"),
                tags: vec!["bench".into()],
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
            .collect();

        let index = ContactsIndex::build(&contacts);
        let query = "Person1234";

        let start = Instant::now();
        let _results = index.fuzzy_search_concurrency(query, &contacts, 1);
        let duration = start.elapsed();

        println!(
            "Concurrent fuzzy search completed in: {:?} ({} contacts)",
            duration,
            contacts.len()
        );

        // assert!(duration.as_secs_f64() < 0.5, "Search took too long!");
    }
    #[test]
    fn benchmark_fuzzy_search() {
        // Generate 10,000 fake contacts
        let contacts: Vec<_> = (0..10_000)
            .map(|i| Contact {
                name: format!("Person{}", i),
                email: format!("person{}@mail.com", i),
                phone: format!("232323323211"),
                tags: vec!["bench".into()],
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
            .collect();

        let index = ContactsIndex::build(&contacts);
        let query = "Person1234";

        let start = Instant::now();
        let _results = index.fuzzy_search(query, &contacts, 2);
        let duration = start.elapsed();

        println!(
            "Normal fuzzy search completed in: {:?} ({} contacts)",
            duration,
            contacts.len()
        );
    }

    #[test]
    fn benchmark_name_lookup() {
        // Generate 10,000 fake contacts
        let contacts: Vec<_> = (0..10_000)
            .map(|i| Contact {
                name: format!("Person{}", i),
                email: format!("person{}@mail.com", i),
                phone: format!("232323323211"),
                tags: vec!["bench".into()],
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
            .collect();

        let index = ContactsIndex::build(&contacts);
        let query = "Person1234";

        let start = Instant::now();
        let _results = index.lookup_name(query);
        let duration = start.elapsed();

        println!(
            "Name lookup search completed in: {:?} ({} contacts)",
            duration,
            contacts.len()
        );
    }
    #[test]
    fn benchmark_domain_lookup() {
        // Generate 10,000 fake contacts
        let contacts: Vec<_> = (0..10_000)
            .map(|i| Contact {
                name: format!("Person{}", i),
                email: format!("person{}@work.com", i),
                phone: format!("232323323211"),
                tags: vec!["bench".into()],
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
            .collect();

        let index = ContactsIndex::build(&contacts);
        let query = "work.com";

        let start = Instant::now();
        let _results = index.lookup_domain(query);
        let duration = start.elapsed();

        println!(
            "Domain lookup search completed in: {:?} ({} contacts)",
            duration,
            contacts.len()
        );
    }
}
