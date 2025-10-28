#![allow(dead_code)]

use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    path::Path,
    sync::{Arc, Mutex},
    thread,
};

use chrono::{DateTime, Utc};
use csv::ReaderBuilder;
use fuzzy_search::distance::levenshtein;

use crate::{domain::Contact, traits::ContactStore};

impl Contact {
    pub fn new(
        name: &str,
        phone: &str,
        email: &str,
        tags: Vec<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            name: name.to_string(),
            phone: phone.to_string(),
            email: email.to_string(),
            tags,
            created_at,
            updated_at,
        }
    }

    // for migrating from txt -> JSON
    pub fn from_line(line: &str) -> Result<Self, AppError> {
        let parts: Vec<&str> = line.split(",").collect();
        if parts.len() != 3 {
            return Err(AppError::Parse(format!("Invalid line: {}", line)));
        }
        Ok(Self::new(
            parts[0],
            parts[1],
            parts[2],
            vec![],
            Utc::now(),
            Utc::now(),
        ))
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    pub fn has_domain(&self, domain: &str) -> bool {
        self.email.ends_with(&format!("@{}", domain))
    }
}

const JSON_FILE_PATH: &str = "contacts.json";
const TXT_FILE_PATH: &str = "contacts.txt";
const CSV_FILE_PATH: &str = "contacts.csv";

#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    Parse(String),
    Validation(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Io(err) => write!(f, "I/O error: {}", err),
            AppError::Parse(msg) => write!(f, "Parse error: {}", msg),
            AppError::Validation(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

//File-based storage

pub struct FileStore;

// impl FileStore {
//     pub fn new(path: &str) -> Self {
//         Self {
//             path: path.to_string(),
//         }
//     }
// }

impl ContactStore for FileStore {
    fn load(&self) -> Result<Vec<Contact>, AppError> {
        let path_json = Path::new(JSON_FILE_PATH);
        let path_txt = Path::new(TXT_FILE_PATH);
        let path_csv = Path::new(CSV_FILE_PATH);

        if path_json.exists() {
            let data = fs::read_to_string(path_json)?;

            let contacts: Vec<Contact> = serde_json::from_str(&data)
                .map_err(|e| AppError::Parse(format!("Error, JSON... : {}", e)))?;
            Ok(contacts)
        } else if path_txt.exists() {
            //txt -> JSON
            let data = fs::read_to_string(path_txt)?;
            let mut contacts = Vec::new();

            for line in data.lines() {
                match Contact::from_line(line) {
                    Ok(c) => contacts.push(c),
                    Err(e) => eprintln!("⚠️ Skipping bad line: {}", e),
                }
            }
            self.save(&contacts)?;
            println!("Migration contacts.txt -> contact.json successful!");
            fs::remove_file(TXT_FILE_PATH)?;
            Ok(contacts)
        } else if path_csv.exists() {
            //CSV -> JSON
            let file = File::open(path_csv)?;
            let mut rdr = ReaderBuilder::new().from_reader(file);
            let mut contacts = Vec::new();

            for result in rdr.deserialize() {
                let contact: Contact = result.map_err(|e| AppError::Parse(e.to_string()))?;
                contacts.push(contact);
            }
            Ok(contacts)
        } else {
            Ok(Vec::new())
        }
    }

    fn save(&self, contacts: &[Contact]) -> Result<(), AppError> {
        let data = serde_json::to_string_pretty(contacts)
            .map_err(|e| AppError::Parse(format!("Saving error...: {}", e)))?;
        fs::write(JSON_FILE_PATH, data)?;
        Ok(())
    }

    fn search(&self, name: String, domain: String, fuzzy: String) -> Result<Vec<usize>, AppError> {
        let contacts = self.load()?;

        let index = ContactsIndex::build(&contacts);

        let mut matches: Vec<usize> = Vec::new();

        if !name.is_empty() {
            matches.extend(index.lookup_name(&name));
        }

        if !domain.is_empty() {
            matches.extend(index.lookup_domain(&domain));
        }

        if !fuzzy.is_empty() {
            matches.extend(index.fuzzy_search(&fuzzy, &contacts, 2))
        }

        println!("Matches {:?}", matches);

        if matches.is_empty() {
            println!("No contacts matched your search.")
            // return Ok(());
        }

        println!("Found {} result(s)", matches.len());
        for i in matches.clone() {
            if let Some(c) = contacts.get(i) {
                println!(
                    "- {} - {} - {} - [{}]",
                    c.name,
                    c.phone,
                    c.email,
                    c.tags.join(", ")
                )
            }
        }
        Ok(matches)
    }
    fn merge_from_file(&self, other_path: &str, policy: MergePolicy) -> Result<(), AppError> {
        let mut existing_contacts = self.load()?;
        let data = fs::read_to_string(other_path)?;

        let imported_contacts: Vec<Contact> = serde_json::from_str(&data)
            .map_err(|e| AppError::Parse(format!("Error, JSON... : {}", e)))?;

        let mut existing_keys: HashSet<(String, String)> = existing_contacts
            .iter()
            .map(|c| (c.name.clone(), c.email.clone()))
            .collect();

        for mut contact in imported_contacts {
            let key = (contact.name.clone(), contact.email.clone());

            match policy {
                MergePolicy::Keep => {
                    if existing_keys.contains(&key) {
                        continue;

                    } else {
                        existing_keys.insert(key);
                        existing_contacts.push(contact);
                    }
                }
                MergePolicy::Overwrite => {
                    if let Some(pos) = existing_contacts.iter().position(|c|c.name == key.0 && c.email == key.1) {
                        existing_contacts[pos] = contact;
                    } else {
                        existing_contacts.push(contact);
                    }
                }
                MergePolicy::Duplicate => {
                    if existing_keys.contains(&key) {

                        contact.name = format!("{} (dup)", contact.name);
                    }
                    existing_keys.insert((contact.name.clone(), contact.email.clone()));
                    existing_contacts.push(contact);
                }
            }
        }
        self.save(&existing_contacts)?;
        Ok(())
    }
}

//Memory storage

pub struct MemStore {
    contacts: std::cell::RefCell<Vec<Contact>>,
}

#[allow(clippy::new_without_default)]
impl MemStore {
    pub fn new() -> Self {
        Self {
            contacts: std::cell::RefCell::new(Vec::new()),
        }
    }
}

impl ContactStore for MemStore {
    fn load(&self) -> Result<Vec<Contact>, AppError> {
        Ok(self.contacts.borrow().clone())
    }

    fn save(&self, contacts: &[Contact]) -> Result<(), AppError> {
        *self.contacts.borrow_mut() = contacts.to_vec();
        Ok(())
    }
    fn search(&self, name: String, domain: String, fuzzy: String) -> Result<Vec<usize>, AppError> {
        let contacts = self.load()?;
        let index = ContactsIndex::build(&contacts);

        let mut matches: Vec<usize> = Vec::new();

        if !name.is_empty() {
            matches.extend(index.lookup_name(&name));
        }

        if !domain.is_empty() {
            matches.extend(index.lookup_domain(&domain));
        }

        if !fuzzy.is_empty() {
            matches.extend(index.fuzzy_search(&fuzzy, &contacts, 2));
        }

        matches.sort_unstable();
        matches.dedup();

        if matches.is_empty() {
            println!("No contacts matched your search.");
        } else {
            println!("Found {} result(s)", matches.len());
            for i in &matches {
                if let Some(c) = contacts.get(*i) {
                    println!(
                        "- {} - {} - {} - [{}]",
                        c.name,
                        c.phone,
                        c.email,
                        c.tags.join(", ")
                    );
                }
            }
        }
        Ok(matches)
    }
    fn merge_from_file(&self, other_path: &str, policy: MergePolicy) -> Result<(), AppError> {
        let mut existing_contacts = self.load()?;
        let data = fs::read_to_string(other_path)?;

        let imported_contacts: Vec<Contact> = serde_json::from_str(&data)
            .map_err(|e| AppError::Parse(format!("Error, JSON... : {}", e)))?;

        let mut existing_keys: HashSet<(String, String)> = existing_contacts
            .iter()
            .map(|c| (c.name.clone(), c.email.clone()))
            .collect();

        for mut contact in imported_contacts {
            let key = (contact.name.clone(), contact.email.clone());

            match policy {
                MergePolicy::Keep => {
                    if existing_keys.contains(&key) {
                        continue;

                    } else {
                        existing_keys.insert(key);
                        existing_contacts.push(contact);
                    }
                }
                MergePolicy::Overwrite => {
                    if let Some(pos) = existing_contacts.iter().position(|c|c.name == key.0 && c.email == key.1) {
                        existing_contacts[pos] = contact;
                    } else {
                        existing_contacts.push(contact);
                    }
                }
                MergePolicy::Duplicate => {
                    if existing_keys.contains(&key) {

                        contact.name = format!("{} (dup)", contact.name);
                    }
                    existing_keys.insert((contact.name.clone(), contact.email.clone()));
                    existing_contacts.push(contact);
                }
            }
        }
        self.save(&existing_contacts)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ContactsIndex {
    name_map: HashMap<String, Vec<usize>>,
    domain_map: HashMap<String, Vec<usize>>,
}

impl ContactsIndex {
    pub fn build(contacts: &[Contact]) -> Self {
        let mut name_map: HashMap<String, Vec<usize>> = HashMap::new();
        let mut domain_map: HashMap<String, Vec<usize>> = HashMap::new();

        for (i, c) in contacts.iter().enumerate() {
            let name_key = c.name.to_lowercase();
            name_map.entry(name_key).or_default().push(i);

            if let Some(domain) = c.email.split('@').nth(1) {
                let domain_key = domain.to_lowercase();
                domain_map.entry(domain_key).or_default().push(i);
            }
        }

        ContactsIndex {
            name_map,
            domain_map,
        }
    }

    pub fn lookup_name(&self, name: &str) -> Vec<usize> {
        let key = name.to_lowercase();
        self.name_map.get(&key).cloned().unwrap_or_default()
    }

    pub fn lookup_domain(&self, domain: &str) -> Vec<usize> {
        let key = domain.to_lowercase();
        self.domain_map.get(&key).cloned().unwrap_or_default()
    }

    pub fn fuzzy_search(&self, query: &str, contacts: &[Contact], max_edits: usize) -> Vec<usize> {
        println!("Running fuzzy search...");
        let q = query.to_lowercase();
        let mut results = Vec::new();

        for (i, c) in contacts.iter().enumerate() {
            let name_distance = levenshtein(&q, &c.name.to_lowercase());
            let email_distance = levenshtein(&q, &c.email.to_lowercase());

            if name_distance < max_edits || email_distance < max_edits {
                results.push(i);
            }
        }

        results
    }

    pub fn fuzzy_search_concurrency(
        &self,
        query: &str,
        contacts: &[Contact],
        max_edits: usize,
    ) -> Vec<usize> {
        println!("Running fuzzy search with concurrency...");
        let num_threads = 4;
        // let size_of_chunk = (contacts.len() + num_threads - 1) /num_threads;
        let size_of_chunk = contacts.len().div_ceil(num_threads).div_ceil(num_threads);
        let query = query.to_lowercase();

        //Sharing data between threads
        let contacts_arc = Arc::new(contacts.to_vec());
        let results = Arc::new(Mutex::new(Vec::new()));

        let mut handles = Vec::new();

        //Splitting into Chunks -> Spawn threads
        for start_of_chunk in (0..contacts.len()).step_by(size_of_chunk) {
            let end_of_chunk = usize::min(start_of_chunk + size_of_chunk, contacts.len());
            let chunk_contacts = contacts_arc.clone();
            let query_clone = query.clone();
            let results_clone = Arc::clone(&results);

            let handle = thread::spawn(move || {
                let mut local_results = Vec::new();

                //Each threads going to work!
                for (i, c) in chunk_contacts[start_of_chunk..end_of_chunk]
                    .iter()
                    .enumerate()
                {
                    let name_distance = levenshtein(&query_clone, &c.name.to_lowercase());
                    let email_distance = levenshtein(&query_clone, &c.email.to_lowercase());

                    if name_distance <= max_edits || email_distance <= max_edits {
                        local_results.push(start_of_chunk + i);
                    }
                }

                // Merge partial results
                let mut global_results = results_clone.lock().unwrap();
                global_results.extend(local_results);
            });

            handles.push(handle);
        }

        // Waiting for all the threads to complete
        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        // Get final results
        Arc::try_unwrap(results).unwrap().into_inner().unwrap()
    }
}

#[derive(Debug, Clone)]
pub enum MergePolicy {
    Keep,
    Overwrite,
    Duplicate,
}

impl MergePolicy {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "overwrite" => MergePolicy::Overwrite,
            "duplication" => MergePolicy::Duplicate,
            _ => MergePolicy::Keep,
        }
    }
}
