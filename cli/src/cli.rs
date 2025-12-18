use chrono::Utc;
use clap::{Parser, Subcommand};
use rolodex_core::domain::{Contact, Contacts, export_csv, import_csv};
use rolodex_core::error::AppError;
use rolodex_core::store::{ContactStore, FileStore, MergePolicy};
use rolodex_core::validation::{
    ValidationResponse, validate_email, validate_name, validate_phone_number,
};
use uuid::Uuid;
use std::env;

// use crate::domain::{Contact, Contacts, export_csv, import_csv};
// use crate::store::mem::{AppError, FileStore, MemStore, MergePolicy};
// use crate::validation::{ValidationResponse, validate_email, validate_name, validate_phone_number};

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
        id: Uuid,
        // #[arg(long)]
        // phone: Option<String>,
    },
    Update {
        #[arg(long)]
        id: Uuid,
        // #[arg(long)]
        // phone: String,
        // #[arg(long, value_delimiter = ',')]
        // tags: Vec<String>,
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
        #[arg(long)]
        concurrent: Option<String>,
    },
    Sync {
        #[arg(long)]
        file: String,
        #[arg(long, default_value = "keep")]
        policy: String,
    },
    Export {
        #[arg(long)]
        to: String,
    },
    Import {
        #[arg(long)]
        from: String,
    },
}

// fn get_store() -> Box<dyn ContactStore> {
//     match env::var("STORE_TYPE")
//         .unwrap_or("file".to_string())
//         .as_str()
//     {
//         "mem" => Box::new(MemStore::new()),
//         _ => Box::new(FileStore),
//     }
// }

fn get_store() -> Box<dyn ContactStore> {
    // match env::var("STORE_TYPE")
    //     .unwrap_or("file".to_string())
    //     .as_str()
    // {
    //     "mem" => Box::new(),
    //     _ => Box::new(FileStore::new("contacts.json")),
    // }

    let _ = env::var("STORE_TYPE")
        .unwrap_or("file".to_string())
        .as_str();
    Box::new(FileStore::new("contacts.json"))
}

pub async fn run_command_cli() -> Result<(), AppError> {
    let cli = Cli::parse();
    let store = get_store();
    // let store = FsStore::new("contacts.json");

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
            store.save(contacts.items.clone())?;
            println!("✅ Added contact: {} ({})", name, email);
        }
        Commands::List { sort, tag, domain } => {

            //Chain filters using iterator
            let mut filtered_contacts: Vec<&Contact> = contacts
                .iter()
                .filter(|c| tag.as_ref().is_none_or(|t| c.has_tag(t)))
                .filter(|c| domain.as_ref().is_none_or(|d| c.has_domain(d)))
                .collect();

            filtered_contacts.sort_by(|a, b| a.name.cmp(&b.name));

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
                    println!("📇 {} | {} | {}", c.name, c.phone.join(", "), c.email);
                }
            }
        }
        Commands::Delete { id } => {
            contacts.delete(id)?;
            store.save(contacts.items.clone())?;
        }
        Commands::Update {
            id,
            new_name,
            new_phone,
            new_email,
        } => {
            contacts.update(id, new_name, new_phone, new_email)?;
            store.save(contacts.items.clone())?;
        }
        Commands::ExportCsv { path } => {
            let contacts = store.load()?;

            let vec_contacts: Vec<Contact> = contacts.values().cloned().collect();
            export_csv(&path, &vec_contacts)?;
            println!("✅ Exported {} contacts to {}", contacts.len(), path);
        }
        Commands::ImportCsv { path } => {
            // let mut contacts = store.load()?;
            let imported = import_csv(&path)?;

            // contacts.extend(imported);
            for mut c in imported {
                c.id = Uuid::new_v4();
                contacts.items.insert(c.id, c);
            }
            store.save(contacts.items)?;
            println!("✅ Imported contacts from {}", path);
        }
        Commands::Search {
            name,
            domain,
            fuzzy,
            concurrent
        } => {
            let _d = contacts.search(name, domain, fuzzy, concurrent)?;
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
            // store.merge_from_file(&file, merge_policy)?;
            contacts.merge_from_file(&file, merge_policy)?;
            store.save(contacts.items)?;
            println!("✅ Sync complete using policy: {}", policy);
        }
        Commands::Export { to } => {
            // contacts.export_to_remote(to).await?;
            contacts.async_export(to).await?;
        }
        Commands::Import { from } => {
            contacts.async_check(from).await?;
            store.save(contacts.items)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    // use crate::store::mem::ContactsIndex;

    use rolodex_core::domain::ContactsIndex;
    use uuid::uuid;

    use super::*;

    fn sample_contacts() -> Contacts {
        let mut contacts_hashmap = HashMap::new();

        contacts_hashmap.insert(
            Uuid::parse_str("1").unwrap(),
            Contact::new(
                "Alice",
                "123",
                "alice@work.com",
                vec!["work".into()],
                Utc::now(),
                Utc::now(),
            ),
        );

        contacts_hashmap.insert(
            Uuid::parse_str("2").unwrap(),
            Contact::new(
                "Bob",
                "456",
                "bob@personal.com",
                vec!["personal".into()],
                Utc::now(),
                Utc::now(),
            ),
        );

        contacts_hashmap.insert(
            Uuid::parse_str("3").unwrap(),
            Contact::new(
                "Carol",
                "789",
                "carol@work.org",
                vec!["work".into()],
                Utc::now(),
                Utc::now(),
            ),
        );

        Contacts::new(contacts_hashmap)

        // Contacts::new(vec![
        //     Contact::new(
        //         "1",
        //         "Alice",
        //         "123",
        //         "alice@work.com",
        //         vec!["work".into()],
        //         Utc::now(),
        //         Utc::now(),
        //     ),
        //     Contact::new(
        //         "2",
        //         "Bob",
        //         "456",
        //         "bob@personal.com",
        //         vec!["personal".into()],
        //         Utc::now(),
        //         Utc::now(),
        //     ),
        //     Contact::new(
        //         "3",
        //         "Carol",
        //         "789",
        //         "carol@work.com",
        //         vec!["work".into()],
        //         Utc::now(),
        //         Utc::now(),
        //     ),
        // ])
    }

    fn sample_contacts2() -> HashMap<Uuid, Contact> {
        let mut contacts_hashmap: HashMap<Uuid, Contact> = HashMap::new();

        contacts_hashmap.insert(
            Uuid::parse_str("1").unwrap(),
            Contact::new(
                "Alice",
                "123",
                "alice@work.com",
                vec!["work".into()],
                Utc::now(),
                Utc::now(),
            ),
        );

        contacts_hashmap.insert(
            Uuid::parse_str("2").unwrap(),
            Contact::new(
                "Alicia",
                "123",
                "alicia@work.com",
                vec!["work".into()],
                Utc::now(),
                Utc::now(),
            ),
        );

        contacts_hashmap.insert(
            Uuid::parse_str("3").unwrap(),
            Contact::new(
                "Bob",
                "456",
                "bob@personal.com",
                vec!["personal".into()],
                Utc::now(),
                Utc::now(),
            ),
        );

        contacts_hashmap.insert(
            Uuid::parse_str("4").unwrap(),
            Contact::new(
                "Carol",
                "789",
                "carol@work.com",
                vec!["work".into()],
                Utc::now(),
                Utc::now(),
            ),
        );

        // Contacts::new(contacts_hashmap)
        contacts_hashmap
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
        assert_eq!(work_mails.len(), 1);
    }

    #[test]
    fn test_chainable_filters() {
        let contacts = sample_contacts();
        let results: Vec<_> = contacts
            .iter()
            .filter(|c| c.has_tag("work"))
            .filter(|c| c.has_domain("work.com"))
            // .take(1)
            .collect();
        println!("{:?}", results);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Alice");
    }

    #[test]
    fn test_index_and_lookup() {
        let contacts = sample_contacts2();
        let index = Contacts::new(contacts);

        // let position = index.lookup_name("Alice");
        let dd = index.index.lookup_name("Alice");

        for id in dd {
            let con = index.items.get(&id).unwrap();
            assert_eq!(con.id, Uuid::parse_str("1").unwrap());
        }

        let nn = index.index.lookup_domain("work.com");

        for id in nn {
            let dom = index.items.get(&id).unwrap();
            assert_eq!(dom.email, "alice@work.com");
        }

        // if let Some(id) = index.index.lookup_name("Alice").get("Alice")
        //     && let Some(data) = index.items.get(id)
        // {
        //     assert_eq!(data.id, Uuid::parse_str("1").unwrap());
        // };
        
        // if let Some(id) = index.index.lookup_domain("work.com").get("work.com")
        //     && let Some(data) = index.items.get(id)
        // {
        //     assert_eq!(data.email, "alice@work.com");
        // };

        // let domain_results = index.lookup_domain("work.com");
        //There are two contacts with "work.com" domain: Carol and Alice
        // assert_eq!(domain_results.len(), 3);
    }

    // fuzzy search
    #[test]
    fn test_exact_match_name() {
        let contacts = sample_contacts2();
        let index = ContactsIndex::build(&contacts);
        let results = index.fuzzy_search("Alice", &contacts, 1);
        println!("Results {:?}", results);
        assert_eq!(results.len(), 1);
    }

    // benchmarking
    // 1. Concurrent fuzzy search

    /*

    #[test]
    fn benchmark_fuzzy_search_concurrent() {
        // Generate 10,000 fake contacts
        let contacts: Vec<_> = (0..10_000)
            .map(|i| Contact {
                id: uuid::Uuid::new_v4().to_string(),
                name: format!("Person{}", i),
                email: format!("person{}@mail.com", i),
                phone: vec![format!("232323323211")],
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
                id: uuid::Uuid::new_v4().to_string(),
                name: format!("Person{}", i),
                email: format!("person{}@mail.com", i),
                phone: vec![format!("232323323211")],
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
                id: uuid::Uuid::new_v4().to_string(),
                name: format!("Person{}", i),
                email: format!("person{}@mail.com", i),
                phone: vec![format!("232323323211")],
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
                id: uuid::Uuid::new_v4().to_string(),
                name: format!("Person{}", i),
                email: format!("person{}@work.com", i),
                phone: vec![format!("232323323211")],
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
    } */
}
