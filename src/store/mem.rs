use std::{
    fs,
    io::{self, Write},
    path::Path,
};

use crate::domain::Contact;

impl Contact {
    pub fn new(name: &str, phone: &str, email: &str) -> Self {
        Self {
            name: name.to_string(),
            phone: phone.to_string(),
            email: email.to_string(),
        }
    }
}

const FILE_PATH: &str = "contacts.json";

pub fn retry<F, T>(prompt: &str, f: F) -> T
where
    F: Fn(&str) -> Result<T, AppError>,
{
    loop {
        let mut input = String::new();
        print!("{}", prompt);
        if let Err(e) = io::stdout().flush() {
            eprintln!("Error flushing stdout: {:?}", e);
        }

        if let Err(e) = io::stdin().read_line(&mut input) {
            eprintln!("Error reading input: {:?}", e);
            continue;
        }

        match f(input.trim()) {
            Ok(value) => return value,
            Err(e) => {
                eprintln!("⚠️ {}", e);
                continue;
            }
        }
    }
}

#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Io(err) => write!(f, "I/O error: {}", err),
            AppError::Parse(msg) => write!(f, "Parse error: {}", msg),
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
    fn save(&self, contacts: &Vec<Contact>) -> Result<(), AppError>;
}

//File-based storage

pub struct FileStore {
    path: String,
}

impl FileStore {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
}

impl ContactStore for FileStore {
    fn load(&self) -> Result<Vec<Contact>, AppError> {
        let path = Path::new(FILE_PATH);
        if path.exists() {
            let data = fs::read_to_string(path)?;

            let contacts: Vec<Contact> = serde_json::from_str(&data)
                .map_err(|e| AppError::Parse(format!("Error, JSON... : {}", e)))?;
            Ok(contacts)
        } else {
            Ok(Vec::new())
        }
    }

    fn save(&self, contacts: &Vec<Contact>) -> Result<(), AppError> {
        let data = serde_json::to_string_pretty(contacts)
            .map_err(|e| AppError::Parse(format!("Saving error...: {}", e)))?;
        fs::write(FILE_PATH, data)?;
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

    fn save(&self, contacts: &Vec<Contact>) -> Result<(), AppError> {
        *self.contacts.borrow_mut() = contacts.clone();
        Ok(())
    }
}
