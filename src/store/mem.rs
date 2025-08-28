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

    // pub fn add(&mut self, contact: Contact) {
    //     self.contacts.push(contact);
    // }

    // pub fn list(&self) -> &Vec<Contact> {
    //     &self.contacts
    // }

    // pub fn delete(&mut self, name: &str) -> bool {
    //     let initial_len = self.contacts.len();
    //     self.contacts.retain(|c| c.name.to_lowercase() != name.to_lowercase());
    //     self.contacts.len() < initial_len
    // }
}
