#![allow(dead_code)]

use regex::Regex;

use crate::domain::Contact;

pub enum ValidationResponse {}

impl ValidationResponse {
    pub fn check_name() -> String {
        "Invalid name! Please check the name and try and again".to_string()
    }

    pub fn check_email() -> String {
        "Invalid email! Please check the email and try and again".to_string()
    }

    pub fn check_phone_number() -> String {
        "Invalid phone number! Please check the number and try and again".to_string()
    }
}

pub fn check_contact_exist(contact: &Contact, contact_list: &[Contact]) -> bool {
    contact_list.iter().any(|c| c.name == contact.name)
        && contact_list.iter().any(|c| c.phone == contact.phone)
}

pub fn check_contact_duplicates(name: String, mut contact_list: Vec<Contact>) -> bool {
    // let contacts = vec![contact_list];
    // contacts.retain(|c| );

    contact_list.retain(|c| c.name == name);
    contact_list.len() > 1
}

pub fn validate_name(name: &str) -> bool {
    !name.trim().is_empty() && name.chars().all(|c| c.is_alphabetic() || c.is_whitespace())
}

pub fn validate_phone_number(phone: &str) -> bool {
    let re = Regex::new(r"^\d{10,}$").unwrap();
    re.is_match(phone)
}

pub fn validate_email(email: &str) -> bool {
    let re = Regex::new(r"^[\w\.-]+@[\w\.-]+\.\w+$").unwrap();
    re.is_match(email)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_validate_name() {
        assert!(validate_name("Alice"));
        assert!(!validate_name("123Bob"));
    }

    #[test]
    fn test_validate_phone_number() {
        assert!(validate_phone_number("08123456789"));
        assert!(!validate_phone_number("1234"));
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("user@example.com"));
        assert!(!validate_email("invalid-email"));
    }
}
