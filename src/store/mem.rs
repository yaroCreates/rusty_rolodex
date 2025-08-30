use std::{fs, io::{self, Write}};

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
            AppError::Io(err) =>write!(f, "I/O error: {}", err),
            AppError::Parse(msg) => write!(f, "Parse error: {}", msg),

        }
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