use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
    pub updated_at: DateTime<Utc>
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
