use std::{
    collections::HashMap,
    fs::File,
    sync::{Arc, Mutex},
    thread,
};

use chrono::{DateTime, Utc};
use csv::{ReaderBuilder, Writer};
use fuzzy_search::distance::levenshtein;
use serde::{Deserialize, Serialize};

use crate::prelude::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub name: String,
    pub phone: String,
    pub email: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct Contacts {
    items: Vec<Contact>,
}

impl Contacts {
    pub fn new(items: Vec<Contact>) -> Self {
        Self { items }
    }

    pub fn iter(&'_ self) -> ContactsIter<'_> {
        ContactsIter {
            inner: self.items.iter(),
        }
    }

    // Returns a read-only slice of all contacts.
    pub fn as_slice(&self) -> &[Contact] {
        &self.items
    }

    // Returns a filtered view as a slice (no clones).
    // pub fn filter_by_tag<'a>(&'a self, tag: &str) -> Vec<&'a Contact> {
    //     self.items.iter().filter(|c| c.has_tag(tag)).collect()
    // }

    // pub fn filter_by_domain<'a>(&'a self, domain: &str) -> Vec<&'a Contact> {
    //     self.items.iter().filter(|c| c.has_domain(domain)).collect()
    // }
}

//Return type

pub struct ContactsIter<'a> {
    inner: std::slice::Iter<'a, Contact>,
}

impl<'a> Iterator for ContactsIter<'a> {
    type Item = &'a Contact;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

pub fn export_csv(path: &str, contacts: &[Contact]) -> Result<(), AppError> {
    println!("Export Path: {}", path);
    let file = File::create(path)?;
    let mut wtr = Writer::from_writer(file);
    // println!("Export files: {:?}", wtr.serialize(contacts.first()));
    println!("Export files: {:?}", (contacts.len()));

    for c in contacts {
        wtr.serialize(Some(c))
            .map_err(|e| AppError::Parse(e.to_string()))
            .unwrap();
    }
    wtr.flush()?;
    Ok(())
}

pub fn import_csv(path: &str) -> Result<Vec<Contact>, AppError> {
    let file = File::open(path)?;
    let mut rdr = ReaderBuilder::new().from_reader(file);
    let mut contacts = Vec::new();

    for result in rdr.deserialize() {
        let contact: Contact = result.map_err(|e| AppError::Parse(e.to_string()))?;
        contacts.push(contact);
    }

    if contacts.is_empty() {
        return Err(AppError::Parse("Error: No CSV data".to_string()));
    }
    Ok(contacts)
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
