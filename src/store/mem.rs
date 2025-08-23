use crate::domain::Contact;

pub struct ContactStore {
    pub contacts: Vec<Contact>,
}

impl ContactStore {
    pub fn new() -> Self {
        let mut contacts = Vec::new();
        contacts.push(Contact {
            name: "James Yaro".to_string(),
            phone: "08122121474".to_string(),
            email: "onuhjamesyaro@gmail.com".to_string(),
        });
        Self { contacts }
    }

    pub fn add(&mut self, contact: Contact) {
        self.contacts.push(contact);
    }

    pub fn list(&self) -> &Vec<Contact> {
        &self.contacts
    }

    pub fn delete(&mut self, name: &str) -> bool {
        let initial_len = self.contacts.len();
        self.contacts
            .retain(|c| c.name.to_lowercase() != name.to_lowercase());
        self.contacts.len() < initial_len
    }
}
