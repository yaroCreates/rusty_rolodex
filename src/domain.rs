use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub name: String,
    pub phone: String,
    pub email: String,
    #[serde(default)] 
    pub tags: Vec<String>
}

#[derive(Debug)]
pub struct Contacts {
    items: Vec<Contact>
}

impl Contacts {
    pub fn new(items: Vec<Contact>) -> Self {
        Self {items}
    }

    pub fn iter(&'_ self) -> ContactsIter<'_> {
        ContactsIter {inner: self.items.iter()}
    }
}

pub struct ContactsIter<'a> {
    inner: std::slice::Iter<'a, Contact>,
}

impl<'a> Iterator for ContactsIter<'a> {
    type Item = &'a Contact;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
