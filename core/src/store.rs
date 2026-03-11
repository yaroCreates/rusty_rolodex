use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use chrono::Utc;
use reqwest::{blocking::Client, header::CONTENT_TYPE};
use uuid::Uuid;

use crate::{
    domain::{Contact, ContactRaw},
    error::AppError,
    helpers::get_key,
};

// pub trait ContactStore: Send + Sync {
//     fn load(&self) -> Result<HashMap<Uuid, Contact>, AppError>;
//     fn save(&self, contacts: HashMap<Uuid, Contact>) -> Result<(), AppError>;
//     // fn search(&self, name: String, domain: String, fuzzy: String) -> Result<Vec<usize>, AppError>;
//     // fn merge_from_file(&self, other_path: &str, policy: MergePolicy) -> Result<(), AppError>;
// }
pub trait ContactStore {
    fn load(&self) -> Result<HashMap<Uuid, Contact>, AppError>;
    fn save(&self, contacts: HashMap<Uuid, Contact>) -> Result<(), AppError>;
}

pub struct MemStore {
    contacts: std::cell::RefCell<HashMap<Uuid, Contact>>,
}

#[allow(clippy::new_without_default)]
impl MemStore {
    pub fn new() -> Self {
        let sample_contacts = Contact::new(
            "name",
            "0987654321",
            "name-phone@gmail.com",
            vec![],
            Utc::now(),
            Utc::now(),
        );

        let mut contacts_hashmap: HashMap<Uuid, Contact> = HashMap::new();

        contacts_hashmap.insert(sample_contacts.id, sample_contacts);

        Self {
            // contacts: std::cell::RefCell::new(HashMap::new()),
            contacts: std::cell::RefCell::new(contacts_hashmap),
        }
    }
}

impl ContactStore for MemStore {
    fn load(&self) -> Result<HashMap<Uuid, Contact>, AppError> {
        Ok(self.contacts.borrow().clone())
    }

    fn save(&self, contacts: HashMap<Uuid, Contact>) -> Result<(), AppError> {
        let mut contacts_hashmap = self.contacts.borrow_mut();
        contacts_hashmap.clear();

        for (_key, contact) in contacts {
            contacts_hashmap.insert(contact.id, contact);
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct FileStore {
    path: PathBuf,
    lock: Arc<Mutex<()>>, // file write lock
}

impl FileStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            lock: Arc::new(Mutex::new(())),
        }
    }
}

impl ContactStore for FileStore {
    fn load(&self) -> Result<HashMap<Uuid, Contact>, AppError> {
        if !self.path.exists() {
            return Ok(HashMap::new());
        }
        let data = fs::read_to_string(&self.path)?;

        let contacts: Vec<ContactRaw> = serde_json::from_str(&data)
            .map_err(|e| AppError::Parse(format!("Error, JSON... : {}", e)))?;

        let migrated_contacts: Vec<Contact> = contacts.into_iter().map(Contact::from).collect();

        let mut contacts_hashmap: HashMap<Uuid, Contact> = HashMap::new();
        for contact in migrated_contacts {
            contacts_hashmap.insert(contact.id, contact);
        }
        Ok(contacts_hashmap)
    }

    fn save(&self, contacts: HashMap<Uuid, Contact>) -> Result<(), AppError> {
        let _guard = self.lock.lock();

        let contacts_vec: Vec<Contact> = contacts.values().cloned().collect();

        let data = serde_json::to_string_pretty(&contacts_vec)
            .map_err(|e| AppError::Parse(format!("Saving error...: {}", e)))?;

        fs::write(&self.path, data)?;
        Ok(())
    }
}

pub struct RemoteStore {
    pub remote_url: Option<String>,
    pub remote_url_with_apikey: String,
}

impl RemoteStore {
    pub fn new() -> Self {
        dotenv::dotenv().ok();
        Self {
            remote_url: get_key("REMOTE_URL").ok(),
            remote_url_with_apikey: format!(
                "{}?apiKey={}",
                get_key("REMOTE_URL").unwrap(),
                get_key("REMOTE_API_KEY").unwrap()
            ),
        }
    }
    pub fn import_from_remote(&self, from: String) -> Result<Vec<Contact>, AppError> {
        let client = Client::new();
        let response = client.get(&from).send();

        let data = response?.json::<Vec<Contact>>()?;

        Ok(data)
    }

    pub fn save_to_remote(&self, contacts: HashMap<Uuid, Contact>) -> Result<(), AppError> {
        let contacts_vec: Vec<Contact> = contacts.values().cloned().collect();

        let client = Client::new();

        let response = client
            .put(&self.remote_url_with_apikey)
            .json(&contacts_vec)
            .header(CONTENT_TYPE, "application/json")
            .send()?;

        if response.status() == 200 {
            println!(
                "✅ Successfully saved contacts to {}",
                &self.remote_url_with_apikey
            );
        } else {
            return Err(AppError::Parse("Error accessing remote base".to_string()));
        }
        Ok(())
    }
}

impl Default for RemoteStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ContactStore for RemoteStore {
    fn load(&self) -> Result<HashMap<Uuid, Contact>, AppError> {
        if let Some(url) = &self.remote_url {
            let remote_contacts = self.import_from_remote(url.to_string())?;

            let mut contact_hashmap: HashMap<Uuid, Contact> = HashMap::new();

            // Save contacts to Hashmap
            for contact in remote_contacts {
                contact_hashmap.insert(contact.id, contact);
            }

            Ok(contact_hashmap)
        } else {
            Err(AppError::Parse("Url not found".to_string()))
        }
    }

    fn save(&self, contacts: HashMap<Uuid, Contact>) -> Result<(), AppError> {
        self.save_to_remote(contacts)
    }
}

#[derive(Debug, Clone)]
pub enum MergePolicy {
    Keep,
    Overwrite,
    Duplicate,
}

impl MergePolicy {
    pub fn policy_check(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "overwrite" => MergePolicy::Overwrite,
            "duplicate" => MergePolicy::Duplicate,
            _ => MergePolicy::Keep,
        }
    }
}
