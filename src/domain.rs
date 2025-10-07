use std::fs::File;

use chrono::{DateTime, Utc};
use csv::{ReaderBuilder, Writer};
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
