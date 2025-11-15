use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    sync::{Arc, Mutex},
    thread,
};

use chrono::{DateTime, Utc};
use csv::{ReaderBuilder, Writer};
use fuzzy_search::distance::levenshtein;
use reqwest::{blocking::Client, header::CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use crate::{
    prelude::AppError,
    store::mem::MergePolicy,
    validation::{
        ValidationResponse, check_contact_duplicates, check_contact_exist, validate_email,
        validate_name, validate_phone_number,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub name: String,
    pub phone: Vec<String>,
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
    pub items: Vec<Contact>,
    index: ContactsIndex,
}

#[derive(serde::Deserialize, Debug)]
pub struct JsonBinWrapper {
    pub record: Vec<Contact>,
}

// impl Iterator for Contacts {
//     type Item = &Contact;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.items.next()
//     }
// }

impl Contacts {
    pub fn new(items: Vec<Contact>) -> Self {
        let build_index = ContactsIndex::build(&items);

        Self {
            items,
            index: build_index,
        }
    }

    pub fn iter(&'_ self) -> ContactsIter<'_> {
        ContactsIter {
            inner: self.items.iter(),
        }
    }
    pub fn add(&mut self, contact: Contact) -> Result<(), AppError> {
        if self.check_contact_exist(&contact) {
            return Err(AppError::Validation(
                "Contact with info already exists".to_string(),
            ));
        }
        println!("Index: {:?}", self.index);
        self.items.push(contact);

        let contact_index = self.items.len() - 1;

        let name_key = self.items[contact_index].name.to_lowercase();
        let domain_key = self.items[contact_index]
            .email
            .split("@")
            .nth(1)
            .unwrap_or_default()
            .to_string();

        if let Some(indexes) = self.index.name_map.get_mut(&name_key) {
            indexes.push(contact_index);
        } else {
            self.index.name_map.insert(name_key, vec![contact_index]);
        }

        if let Some(indexes) = self.index.domain_map.get_mut(&domain_key) {
            indexes.push(contact_index);
        } else {
            self.index
                .domain_map
                .insert(domain_key, vec![contact_index]);
        }
        println!("Name index after: {:?}", self.index.name_map);
        println!("Domain index after: {:?}", self.index.domain_map);
        Ok(())
    }

    pub fn delete(&mut self, name: String, phone: Option<String>) -> Result<(), AppError> {
        let phone = phone.unwrap_or_default();

        let name_indexes = self
            .index
            .name_map
            .get(&name.to_lowercase())
            .cloned()
            .unwrap_or_default();
        println!("name_keys: {:?}", name_indexes);

        if name_indexes.is_empty() {
            println!("⚠️ No contact found with name '{}'", name);
            return Ok(());
        }

        if self.check_contact_duplicates(name.clone()) {
            if phone.is_empty() {
                return Err(AppError::Parse(String::from(
                    "There are more than one contact with the name. Please provide the phone number to continue the action",
                )));
            }
            if let Some(index) = self
                .items
                .iter()
                .position(|c| c.name.eq_ignore_ascii_case(&name) && c.phone.contains(&phone))
            {
                self.delete_contact_and_update_indexes(index, &name);
                println!("🗑️ Removed contact: {} - {}", name, phone);
            } else {
                println!(
                    "⚠️ No contact found with name '{}' and phone '{}'",
                    name, phone
                );
            }
        } else if let Some(index) = self
            .items
            .iter()
            .position(|c| c.name.eq_ignore_ascii_case(&name))
        {
            self.delete_contact_and_update_indexes(index, &name);
            println!("🗑️ Removed contact: {}", name);
        } else {
            println!("⚠️ No contact found with name '{}'", name);
        }
        println!("Name index after: {:?}", self.index.name_map);
        println!("Domain index after: {:?}", self.index.domain_map);
        Ok(())
    }
    pub fn update(
        &mut self,
        name: String,
        phone: String,
        tags: Vec<String>,
        new_name: Option<String>,
        new_phone: Option<String>,
        new_email: Option<String>,
    ) -> Result<(), AppError> {
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

        if let Some(index) = self
            .items
            .iter()
            .position(|c| c.name == name && c.phone.contains(&phone))
        {
            let old_name_key = self.items[index].name.to_lowercase();
            let old_domain_key = self.items[index]
                .email
                .split('@')
                .nth(1)
                .unwrap_or_default()
                .to_lowercase();

            if !new_name.is_empty() {
                if !validate_name(&new_name) {
                    return Err(AppError::Validation("New name is invalid".to_string()));
                }
                self.items[index].name = new_name.clone();
            }

            if !new_phone.is_empty() {
                if !validate_phone_number(&new_phone) {
                    return Err(AppError::Validation(
                        "New phone number is invalid".to_string(),
                    ));
                }
                self.items[index].phone = vec![new_phone.clone()];
            }

            if !new_email.is_empty() {
                if !validate_email(&new_email) {
                    return Err(AppError::Validation("New email is invalid".to_string()));
                }
                self.items[index].email = new_email.clone();
            }

            self.items[index].tags = tags;
            self.items[index].updated_at = Utc::now();

            let new_name_key = self.items[index].name.to_lowercase();
            let new_domain_key = self.items[index]
                .email
                .split('@')
                .nth(1)
                .unwrap_or_default()
                .to_lowercase();

            if old_name_key != new_name_key {
                self.update_name_index(index, &old_name_key, &new_name_key);
            }

            if old_domain_key != new_domain_key {
                self.update_domain_index(index, &old_domain_key, &new_domain_key);
            }

            println!(
                "✅ Contact updated: {} - {} -> {} - {} - {}",
                name, phone, new_name, new_phone, new_email
            );
            println!("Updated name index: {:?}", self.index.name_map);
            println!("Updated domain index: {:?}", self.index.domain_map);
        } else {
            return Err(AppError::Parse(format!(
                "Contact with name '{}' and phone '{}' not found",
                name, phone
            )));
        }
        Ok(())
    }
    pub fn import_from_remote(&mut self, from: String) -> Result<(), AppError> {
        let client = Client::new();
        let response = client
            .get(&from)
            .header(
                "X-Master-Key",
                "$2a$10$7oi2iI1oYy/8Y8RuKoS0Auoie61m7Q.lP8rhX0ZLSPsGasxdSzilO",
            )
            .send()?;

        if response.status() == 200 {
            let remote_contacts = response
                .json::<JsonBinWrapper>()
                .map_err(|e| AppError::Parse(format!("Invalid JSON: {}", e)))?;

            for contact in remote_contacts.record {
                self.add(contact)?;
            }
        } else {
            return Err(AppError::Network(format!(
                "Failed to import: {}",
                response.status()
            )));
        }
        Ok(())
    }
    pub fn export_to_remote(self, to: String) -> Result<(), AppError> {
        let client = Client::new();
        let response = client
            .post(&to)
            .json(&self.items)
            .header(CONTENT_TYPE, "application/json")
            .header(
                "X-Master-Key",
                "$2a$10$7oi2iI1oYy/8Y8RuKoS0Auoie61m7Q.lP8rhX0ZLSPsGasxdSzilO",
            )
            .send()?;

        println!("Response from export: {:?}", response);
        // response.status();
        if response.status() == 200 {
            println!(
                "✅ Successfully pushed {} contacts to {}",
                self.items.len(),
                to
            );
        } else {
            println!("Something went wrong!");
        }
        Ok(())
    }

    pub fn merge_from_file(
        &mut self,
        other_path: &str,
        policy: MergePolicy,
    ) -> Result<(), AppError> {
        let data = fs::read_to_string(other_path)?;

        let mut imported_contacts: Vec<Contact> = serde_json::from_str(&data)
            .map_err(|e| AppError::Parse(format!("Error, JSON... : {}", e)))?;

        let mut existing_keys: HashSet<(String, Vec<String>)> = self
            .items
            .iter()
            .map(|c| (c.name.clone(), c.phone.clone()))
            .collect();

        for (i, contact) in imported_contacts.iter_mut().enumerate() {
            let key = (contact.name.clone(), contact.phone.clone());

            // let name_key = contact.name.clone();
            let imported_phone_set: HashSet<_> = contact.phone.iter().collect();

            let domain_key = contact
                .email
                .split("@")
                .nth(1)
                .unwrap_or_default()
                .to_string();

            match policy {
                MergePolicy::Keep => {
                    if existing_keys.contains(&key) {
                        continue;
                    } else {
                        existing_keys.insert(key);
                        self.items.push(contact.clone());
                        self.index.name_map.insert(contact.name.clone(), vec![i]);
                        self.index.domain_map.insert(domain_key, vec![i]);
                    }
                }
                MergePolicy::Overwrite => {
                    if let Some(pos) = self
                        .items
                        .iter()
                        .position(|c| c.name == key.0 && c.phone == key.1)
                    {
                        self.items[pos] = contact.clone();
                    } else {
                        self.items.push(contact.clone());
                        self.index.name_map.insert(contact.name.clone(), vec![i]);
                        self.index.domain_map.insert(domain_key, vec![i]);
                    }
                }
                MergePolicy::Duplicate => {
                    for existing_contact in &mut self.items {
                        if existing_contact.name == contact.name
                            && existing_contact
                                .phone
                                .iter()
                                .any(|p| imported_phone_set.contains(p))
                        {
                            existing_contact.phone.push(contact.phone.join(", "));

                            if let Some(mut key) = existing_keys.take(&key) {
                                key.1.push(contact.phone.join(", "));
                            }
                            existing_contact.updated_at = Utc::now();
                            self.index.name_map.insert(contact.name.clone(), vec![i]);
                            self.index.domain_map.insert(domain_key.clone(), vec![i]);
                        }
                    }
                }
            }
        }
        // self.save(&existing_contacts)?;
        Ok(())
    }

    pub fn check_contact_exist(&self, new_contact: &Contact) -> bool {
        check_contact_exist(new_contact, &self.items)
    }

    pub fn check_contact_duplicates(&self, name: String) -> bool {
        check_contact_duplicates(name, self.items.clone())
    }

    fn delete_contact_and_update_indexes(&mut self, index: usize, name_key: &str) {
        let domain_key = self.items[index]
            .email
            .split('@')
            .nth(1)
            .unwrap_or_default()
            .to_lowercase();

        //Remove the contact itself
        self.items.remove(index);

        //Updating the name_map indexes
        if let Some(indices) = self.index.name_map.get_mut(name_key) {
            indices.retain(|&i| i != index);
            for i in indices.iter_mut() {
                if *i > index {
                    *i -= 1;
                }
            }
            if indices.is_empty() {
                self.index.name_map.remove(name_key);
            }
        }

        // Update domain_map indexes
        if let Some(indices) = self.index.domain_map.get_mut(&domain_key) {
            indices.retain(|&i| i != index);
            for i in indices.iter_mut() {
                if *i > index {
                    *i -= 1;
                }
            }
            if indices.is_empty() {
                self.index.domain_map.remove(&domain_key);
            }
        }
    }

    fn update_name_index(&mut self, index: usize, old_key: &str, new_key: &str) {
        // Remove old index from old name
        if let Some(indices) = self.index.name_map.get_mut(old_key) {
            indices.retain(|&i| i != index);
            if indices.is_empty() {
                self.index.name_map.remove(old_key);
            }
        }

        // Add new index to new name
        if let Some(indices) = self.index.name_map.get_mut(new_key) {
            indices.push(index);
        } else {
            self.index.name_map.insert(new_key.to_string(), vec![index]);
        }
    }

    fn update_domain_index(&mut self, index: usize, old_key: &str, new_key: &str) {
        // Remove old index from old domain
        if let Some(indices) = self.index.domain_map.get_mut(old_key) {
            indices.retain(|&i| i != index);
            if indices.is_empty() {
                self.index.domain_map.remove(old_key);
            }
        }

        // Add new index to new domain
        if let Some(indices) = self.index.domain_map.get_mut(new_key) {
            indices.push(index);
        } else {
            self.index
                .domain_map
                .insert(new_key.to_string(), vec![index]);
        }
    }

    // Returns a read-only slice of all contacts.
    pub fn as_slice(&self) -> &[Contact] {
        &self.items
    }
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
