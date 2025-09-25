#![allow(dead_code)]

use std::{
    fs,
    // io::{self, Write},
    path::Path,
};

use crate::domain::Contact;

impl Contact {
    pub fn new(name: &str, phone: &str, email: &str, tags: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            phone: phone.to_string(),
            email: email.to_string(),
            tags,
        }
    }

    // for migrating from txt -> JSON
    pub fn from_line(line: &str) -> Result<Self, AppError> {
        let parts: Vec<&str> = line.split(",").collect();
        if parts.len() != 3 {
            return Err(AppError::Parse(format!("Invalid line: {}", line)));
        }
        Ok(Self::new(parts[0], parts[1], parts[2], vec![]))
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

pub trait ContactStore {
    fn load(&self) -> Result<Vec<Contact>, AppError>;
    fn save(&self, contacts: &[Contact]) -> Result<(), AppError>;
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
}

//Memory storage

pub struct MemStore {
    contacts: std::cell::RefCell<Vec<Contact>>,
}

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
}
