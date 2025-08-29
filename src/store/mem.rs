use std::{fs, io};

use crate::domain::Contact;

impl Contact {
    pub fn new(name: &str, phone: &str, email: &str) -> Self {
        Self { 
            name: name.to_string(),
            phone: phone.to_string(),
            email: email.to_string()
         }
    }

    pub fn to_line(&self) -> String {
        format!("{},{},{}", self.name, self.phone, self.email)
    }

    pub fn from_line(line: &str) -> Result<Self, AppError> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() == 3 {
            Ok(Self::new(parts[0].trim(), parts[1].trim(), parts[2].trim()))
        } else {
            Err(AppError::Parse(format!("Invalid contact line: {}", line)))
        }
    }
}

#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    Parse(String),
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
        Self { path: path.to_string() }
    }
}

impl ContactStore for FileStore {
    fn load(&self) -> Result<Vec<Contact>, AppError> {
        let data = match fs::read_to_string(&self.path) {
            Ok(d) => d,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(AppError::Io(e)),

        };

        let mut contacts = Vec::new();
        for line in data.lines() {
            match Contact::from_line(line) {
                Ok(c) => contacts.push(c),
                Err(e) => eprintln!("Skipping line: {:?}", e),
            }
        }
        Ok(contacts)
    }

    fn save(&self, contacts: &Vec<Contact>) -> Result<(), AppError>{
        let data = contacts.iter().map(|c| c.to_line()).collect::<Vec<_>>().join("\n");
        fs::write(&self.path, data).expect("Failed to save contacts");
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