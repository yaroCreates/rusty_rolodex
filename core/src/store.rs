use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use uuid::Uuid;

use crate::{domain::{Contact, ContactRaw}, error::AppError};

// const JSON_FILE_PATH: &str = "contacts.json";

pub trait ContactStore: Send + Sync {
    fn load(&self) -> Result<HashMap<Uuid, Contact>, AppError>;
    fn save(&self, contacts: HashMap<Uuid, Contact>) -> Result<(), AppError>;
    // fn search(&self, name: String, domain: String, fuzzy: String) -> Result<Vec<usize>, AppError>;
    // fn merge_from_file(&self, other_path: &str, policy: MergePolicy) -> Result<(), AppError>;
}

// pub struct MemStore {
//     contacts: std::cell::RefCell<HashMap<String, Contact>>,
// }

// #[allow(clippy::new_without_default)]
// impl MemStore {
//     pub fn new() -> Self {
//         Self {
//             contacts: std::cell::RefCell::new(HashMap::new()),
//         }
//     }
// }

// impl ContactStore for MemStore {
//     fn load(&self) -> Result<HashMap<String, Contact>, AppError> {
//         Ok(self.contacts.borrow().clone())
//     }

//     fn save(&self, contacts: HashMap<String, Contact>) -> Result<(), AppError> {
//         let mut contacts_hashmap = self.contacts.borrow_mut();
//         contacts_hashmap.clear();

//         for (_key, contact) in contacts {
//             contacts_hashmap.insert(contact.id.clone(), contact.clone());
//         }
//         Ok(())
//     }
// }

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
        // let path_json = Path::new(JSON_FILE_PATH);
        if !self.path.exists() {
            return Ok(HashMap::new());
        }
        let data = fs::read_to_string(&self.path)?;
        // let contacts: Vec<Contact> = serde_json::from_str(&data)
        //     .map_err(|e| AppError::Parse(format!("Error, JSON... : {}", e)))?;
        let contacts: Vec<ContactRaw> = serde_json::from_str(&data)
            .map_err(|e| AppError::Parse(format!("Error, JSON... : {}", e)))?;

        let migrated_contacts:Vec<Contact> = contacts.into_iter().map(Contact::from).collect();

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
