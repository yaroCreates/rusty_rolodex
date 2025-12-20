use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

// use crate::{domain::Contact, prelude::AppError};

use std::{
    collections::{HashMap, HashSet},
    env,
    fs::File,
    sync::{Arc, Mutex},
    thread,
};

use chrono::{DateTime, Utc};
use csv::{ReaderBuilder, Writer};
use dotenv::dotenv;
use fuzzy_search::distance::levenshtein;
use reqwest::{Client, header::CONTENT_TYPE};

use crate::{error::AppError, store::MergePolicy, validation::ValidationResponse};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub path: String,
}

impl AppState {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }

    pub fn load(&self) -> Result<Vec<Contact>, AppError> {
        let data = fs::read_to_string(&self.path)
            .map_err(|e| AppError::Parse(format!("Read error: {}", e)))?;

        let contacts: Vec<Contact> = serde_json::from_str(&data)
            .map_err(|e| AppError::Parse(format!("JSON error: {}", e)))?;

        Ok(contacts)
    }

    pub fn save(&self, contacts: &[Contact]) -> Result<(), AppError> {
        let serialized = serde_json::to_string_pretty(contacts)
            .map_err(|e| AppError::Parse(format!("Serialize error: {}", e)))?;

        fs::write(&self.path, serialized)
            .map_err(|e| AppError::Parse(format!("Write error: {}", e)))?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactRaw {
    pub id: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: Uuid,
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
            id: Uuid::new_v4(),
            name: name.to_string(),
            phone: vec![phone.to_string()],
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
    pub fn from(raw: ContactRaw) -> Self {
        let id = Uuid::parse_str(&raw.id).unwrap_or_else(|_| Uuid::new_v4());

        Contact {
            id,
            name: raw.name,
            phone: raw.phone,
            email: raw.email,
            tags: raw.tags,
            created_at: raw.created_at,
            updated_at: raw.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialContact {
    pub name: String,
    pub phone: Vec<String>,
    pub email: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub updated_at: DateTime<Utc>,
    pub id: Option<String>,
}

#[derive(Debug)]
pub struct Contacts {
    pub items: HashMap<Uuid, Contact>,
    pub index: ContactsIndex,
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
    pub fn new(items: HashMap<Uuid, Contact>) -> Self {
        let build_index = ContactsIndex::build(&items);

        Self {
            items,
            index: build_index,
        }
    }

    pub fn iter(&'_ self) -> ContactsIter<'_> {
        ContactsIter {
            inner: self.items.values(),
        }
    }

    pub fn add_index(&mut self, contact: &Contact) {
        let name_key = contact.name.to_lowercase();
        self.index
            .name_map
            .entry(name_key)
            .or_default()
            .insert(contact.id);

        if let Some(domain) = contact.email.split("@").nth(1) {
            self.index
                .domain_map
                .entry(domain.to_lowercase())
                .or_default()
                .insert(contact.id);
        }
    }

    pub fn remove_index(&mut self, contact: &Contact) {
        let name_key = contact.name.to_lowercase();

        if let Some(set) = self.index.name_map.get_mut(&name_key) {
            set.remove(&contact.id);
        }

        if let Some(domain) = contact.email.split("@").nth(1)
            && let Some(set) = self.index.domain_map.get_mut(&domain.to_lowercase())
        {
            set.remove(&contact.id);
        }

        let _ = &self.index.name_map.retain(|_key, set| !set.is_empty());

        let _ = &self.index.domain_map.retain(|_key, set| !set.is_empty());
    }

    pub fn update_index(&mut self, id: Uuid) {
        let contact = self.items.get(&id).unwrap();

        self.index
            .name_map
            .entry(contact.name.clone())
            .or_default()
            .insert(id);

        let domain = contact.email.split("@").nth(1).unwrap().to_string();
        self.index.domain_map.entry(domain).or_default().insert(id);
    }

    pub fn find_with_name_phone(&self, name: &str, phone: &[String]) -> Option<Uuid> {
        let imported_phone_set: HashSet<_> = phone.iter().collect();

        self.index.name_map.get(name).and_then(|ids| {
            ids.iter()
                .find(|&id| {
                    if let Some(c) = self.items.get(id) {
                        c.phone.iter().any(|p| imported_phone_set.contains(p))
                    } else {
                        false
                    }
                })
                .cloned()
        })
    }

    pub fn add(&mut self, mut contact: Contact) -> Result<(), AppError> {
        if self.check_contact_exist(&contact) {
            return Err(AppError::Validation(
                "Contact with info already exists".to_string(),
            ));
        }
        println!("Index: {:?}", self.index);
        contact.id = Uuid::new_v4();

        self.items.insert(contact.id, contact.clone());
        self.add_index(&contact);

        println!("Name index after: {:?}", self.index.name_map);
        println!("Domain index after: {:?}", self.index.domain_map);
        Ok(())
    }

    pub fn delete(&mut self, id: Uuid) -> Result<(), AppError> {
        let contact = self
            .items
            .remove(&id)
            .ok_or(AppError::Parse("No contact found".to_string()))?;
        self.remove_index(&contact);

        println!("Name index after: {:?}", self.index.name_map);
        println!("Domain index after: {:?}", self.index.domain_map);

        Ok(())
    }
    fn check_contact_before_updating(
        &self,
        contact: &Contact,
        new_name: &Option<String>,
        new_email: &Option<String>,
    ) -> Result<(), AppError> {
        if let Some(n) = new_name.clone()
            && n == contact.name
        {
            return Err(AppError::Validation(
                "Contact already has same name".to_string(),
            ));
        }

        if let Some(e) = new_email.clone()
            && e == contact.email
        {
            return Err(AppError::Validation(
                "Contact already has same email address".to_string(),
            ));
        }
        Ok(())
    }
    pub fn update(
        &mut self,
        id: Uuid,
        new_name: Option<String>,
        new_phone: Option<String>,
        new_email: Option<String>,
    ) -> Result<(), AppError> {
        if id.is_nil() {
            return Err(AppError::Validation(ValidationResponse::check_uuid()));
        }

        let mut contact = self
            .items
            .get(&id)
            .cloned()
            .ok_or(AppError::Parse("Contact not found".to_string()))?;

        self.check_contact_before_updating(&contact, &new_name, &new_email)?;

        self.remove_index(&contact);

        if let Some(name) = new_name {
            contact.name = name
        }
        if let Some(phone) = new_phone {
            contact.phone = vec![phone]
        }
        if let Some(email) = new_email {
            contact.email = email
        }
        contact.updated_at = Utc::now();

        self.items.insert(contact.id, contact.clone());

        self.add_index(&contact);

        println!("✅ Contact updated");

        println!("Updated name index: {:?}", self.index.name_map);
        println!("Updated domain index: {:?}", self.index.domain_map);

        Ok(())
    }

    pub fn search(
        &self,
        name: Option<String>,
        domain: Option<String>,
        fuzzy: Option<String>,
        concurrent: Option<String>,
    ) -> Result<Vec<Contact>, AppError> {
        let mut matches: Vec<Contact> = Vec::new();

        if let Some(name) = name {
            let ids = self.index.lookup_name(&name);
            for id in ids {
                let result = self.items.get(&id).expect("Error");
                matches.push(result.clone());
            }
        }

        if let Some(domain) = domain {
            let ids = self.index.lookup_domain(&domain);
            for id in ids {
                let result = self.items.get(&id).expect("Error");
                matches.push(result.clone());
            }
        }

        if let Some(fuzzy) = fuzzy {
            let g = self.index.fuzzy_search(&fuzzy, &self.items, 2);

            for item in g {
                matches.push(item.clone());
            }
        }

        if let Some(concurrent) = concurrent {
            let g = self
                .index
                .fuzzy_search_concurrency(&concurrent, &self.items, 2);

            for item in g {
                matches.push(item.clone());
            }
        }

        println!("Matches {:?}", matches);

        if matches.is_empty() {
            println!("No contacts matched your search.")
            // return Ok(());
        }

        println!("Found {} result(s)", matches.len());
        for i in matches.clone() {
            // if let Some(c) = contacts.get(i) {
            println!(
                "- {} - [{}] - {} - [{}]",
                i.name,
                i.phone.join(", "),
                i.email,
                i.tags.join(", ")
            )
            // }
        }
        Ok(matches)
    }

    pub async fn import_from_remote(&mut self, from: String) -> Result<(), AppError> {
        let client = Client::new();
        let response = client
            .get(&from)
            .header(
                "X-Master-Key",
                "$2a$10$7oi2iI1oYy/8Y8RuKoS0Auoie61m7Q.lP8rhX0ZLSPsGasxdSzilO",
            )
            .send()
            .await?;

        if response.status() == 200 {
            let remote_contacts = response
                .json::<JsonBinWrapper>()
                .await
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

    pub async fn export_to_remote(self, to: String) -> Result<(), AppError> {
        dotenv().ok();

        let master_key = env::var("MASTER_KEY").expect("Error: Master key must be set");

        println!("Master_key: {}", master_key);

        let client = Client::new();
        let response = client
            .post(&to)
            .json(&self.items)
            .header(CONTENT_TYPE, "application/json")
            .header(
                "X-Master-Key",
                "$2a$10$7oi2iI1oYy/8Y8RuKoS0Auoie61m7Q.lP8rhX0ZLSPsGasxdSzilO",
            )
            .send()
            .await?;

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

    pub async fn async_export(self, to: String) -> Result<(), AppError> {
        let client = Client::new();
        let response = client
            .post(&to)
            .json(&self.items)
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await?;

        let res = response.json::<Vec<SpecialContact>>().await?;
        println!("Response from export: {:?}", res);

        let data = serde_json::to_string_pretty(&res)
            .map_err(|e| AppError::Parse(format!("Saving error...: {}", e)))?;
        fs::write("export_contacts.json", data)?;

        Ok(())
    }

    pub async fn async_check(&mut self, from: String) -> Result<(), AppError> {
        let client = Client::new();
        let response = client
            .get(&from)
            // .header(
            //     "X-Master-Key",
            //     "$2a$10$7oi2iI1oYy/8Y8RuKoS0Auoie61m7Q.lP8rhX0ZLSPsGasxdSzilO",
            // )
            .send()
            .await?;

        // let response = reqwest::get(from).await?;

        let data = response.json::<Vec<Contact>>().await;
        println!("This is the response: {:?}", data);

        for contact in data? {
            self.add(contact)?;
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
            .map(|c| (c.1.name.clone(), c.1.phone.clone()))
            .collect();

        for contact in imported_contacts.iter_mut() {
            let key = (contact.name.clone(), contact.phone.clone());

            // let name_key = contact.name.clone();
            // let imported_phone_set: HashSet<_> = contact.phone.iter().collect();

            // let domain_key = contact
            //     .email
            //     .split("@")
            //     .nth(1)
            //     .unwrap_or_default()
            //     .to_string();

            match policy {
                MergePolicy::Keep => {
                    if existing_keys.contains(&key) {
                        continue;
                    } else {
                        existing_keys.insert(key);
                        contact.id = Uuid::new_v4();

                        self.items.insert(contact.id, contact.clone());
                        self.add_index(contact);
                    }
                }
                MergePolicy::Overwrite => {
                    if let Some(existing_id) =
                        self.find_with_name_phone(&contact.name, &contact.phone)
                    {
                        contact.id = existing_id;
                        self.items.insert(existing_id, contact.clone());
                        self.remove_index(contact);
                        self.add_index(contact);
                    } else {
                        contact.id = Uuid::new_v4();
                        self.items.insert(contact.id, contact.clone());
                        self.update_index(contact.id);
                    }
                }
                MergePolicy::Duplicate => {
                    for (_key, mut existing_contact) in self.items.clone() {
                        if self
                            .find_with_name_phone(&contact.name, &contact.phone)
                            .is_some()
                        {
                            existing_contact.phone.push(contact.phone.join(", "));
                        }

                        contact.id = Uuid::new_v4();

                        self.items.insert(contact.id, contact.clone());
                        self.add_index(contact);
                    }
                }
            }
        }
        // self.save(&existing_contacts)?;
        Ok(())
    }

    pub fn check_contact_exist(&self, new_contact: &Contact) -> bool {
        let imported_phone_set: HashSet<_> = new_contact.phone.iter().collect();

        let get_uuids = self.index.name_map.get(&new_contact.name).and_then(|ids| {
            ids.iter()
                .find(|&id| {
                    if let Some(c) = self.items.get(id) {
                        c.phone.iter().any(|p| imported_phone_set.contains(p))
                    } else {
                        false
                    }
                })
                .cloned()
        });

        // if let Some(_status) = get_uuids {
        //     true
        // } else {
        //     false
        // }
        matches!(get_uuids, Some(_status))
    }

    // pub fn check_contact_duplicates(&self, name: String) -> bool {
    //     check_contact_duplicates(name, self.items.clone())
    // }

    // Returns a read-only slice of all contacts.
    // pub fn as_slice(&self) -> &[Contact] {
    //     &self.items
    // }
}

//Return type

pub struct ContactsIter<'a> {
    // inner: std::slice::Iter<'a, Contact>,
    inner: std::collections::hash_map::Values<'a, Uuid, Contact>,
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
    name_map: HashMap<String, HashSet<Uuid>>,
    domain_map: HashMap<String, HashSet<Uuid>>,
}

impl ContactsIndex {
    pub fn build(contacts: &HashMap<Uuid, Contact>) -> Self {
        let mut name_map: HashMap<String, HashSet<Uuid>> = HashMap::new();
        let mut domain_map: HashMap<String, HashSet<Uuid>> = HashMap::new();

        for (_key, contact) in contacts.iter() {
            let name_key = contact.name.to_lowercase();
            name_map.entry(name_key).or_default().insert(contact.id);

            if let Some(domain) = contact.email.split('@').nth(1) {
                let domain_key = domain.to_lowercase();
                domain_map.entry(domain_key).or_default().insert(contact.id);
            }
        }

        ContactsIndex {
            name_map,
            domain_map,
        }
    }

    pub fn lookup_name(&self, name: &str) -> HashSet<Uuid> {
        let key = name.to_lowercase();
        self.name_map.get(&key).cloned().unwrap_or_default()
    }

    pub fn lookup_domain(&self, domain: &str) -> HashSet<Uuid> {
        let key = domain.to_lowercase();
        self.domain_map.get(&key).cloned().unwrap_or_default()
    }

    pub fn fuzzy_search<'a>(
        &self,
        query: &str,
        contacts: &'a HashMap<Uuid, Contact>,
        max_edits: usize,
    ) -> Vec<&'a Contact> {
        println!("Running fuzzy search...");
        let q = query.to_lowercase();
        let mut results: Vec<&Contact> = Vec::new();

        let contacts_x = contacts.values();

        for c in contacts_x {
            let name_distance = levenshtein(&q, &c.name.to_lowercase());
            let email_distance = levenshtein(&q, &c.email.to_lowercase());

            if name_distance < max_edits || email_distance < max_edits {
                results.push(c);
            }
        }

        results
    }

    pub fn fuzzy_search_concurrency(
        &self,
        query: &str,
        contacts: &HashMap<Uuid, Contact>,
        max_edits: usize,
    ) -> Vec<Contact> {
        println!("Running fuzzy search with concurrency...");
        let num_threads = 4;
        let query = query.to_lowercase();

        let contacts_vec: Vec<Contact> = contacts.values().cloned().collect();
        let total = contacts_vec.len();

        if total == 0 {
            return Vec::new();
        }

        // let size_of_chunk = (total + num_threads - 1) /num_threads;
        let size_of_chunk = total.div_ceil(num_threads);
        // let size_of_chunk = contacts.len().div_ceil(num_threads).div_ceil(num_threads);

        //Sharing data between threads
        let contacts_arc = Arc::new(contacts_vec);
        let results = Arc::new(Mutex::new(Vec::new()));

        let mut handles = Vec::new();

        //Splitting into Chunks -> Spawn threads
        for start_of_chunk in (0..total).step_by(size_of_chunk) {
            let end_of_chunk = usize::min(start_of_chunk + size_of_chunk, total);

            let contacts_clone = Arc::clone(&contacts_arc);
            let results_clone = Arc::clone(&results);
            let query_clone = query.clone();

            let handle = thread::spawn(move || {
                let mut local_results = Vec::new();

                //Each threads going to work!
                for c in &contacts_clone[start_of_chunk..end_of_chunk] {
                    let name_distance = levenshtein(&query_clone, &c.name.to_lowercase());
                    let email_distance = levenshtein(&query_clone, &c.email.to_lowercase());

                    if name_distance <= max_edits || email_distance <= max_edits {
                        local_results.push(c.clone());
                    }
                }

                // Merge partial results
                // let mut global_results = results_clone.lock().unwrap();
                // global_results.extend(local_results);
                results_clone.lock().unwrap().extend(local_results);
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
