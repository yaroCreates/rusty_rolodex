use std::fs;

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

    pub fn from_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() == 3 {
            Some(Self::new(parts[0].trim(), parts[1].trim(), parts[2].trim()))
        } else {
            None
        }
    }
}

pub trait ContactStore { 
    fn load(&self) -> Vec<Contact>;
    fn save(&self, contacts: &Vec<Contact>);
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
    fn load(&self) -> Vec<Contact> {
        fs::read_to_string(&self.path)
            .unwrap_or_default()
            .lines()
            .filter_map(Contact::from_line)
            .collect()
    }

    fn save(&self, contacts: &Vec<Contact>) {
        let data = contacts.iter().map(|c| c.to_line()).collect::<Vec<_>>().join("\n");
        fs::write(&self.path, data).expect("Failed to save contacts");
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
    fn load(&self) -> Vec<Contact> {
        self.contacts.borrow().clone()
    }

    fn save(&self, contacts: &Vec<Contact>) {
        *self.contacts.borrow_mut() = contacts.clone();
    }
}