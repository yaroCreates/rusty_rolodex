
pub fn validate_name(name: &str) -> bool {
    !name.trim().is_empty() && name.chars().all(|c| c.is_alphabetic() || c.is_whitespace())
}

pub fn validate_phone(phone: &str) -> bool {
    phone.len() > 10
}

pub fn validate_email(email: &str) -> bool {
    let parts:Vec<&str> = email.split("@").collect();
    if parts.len() != 2 {
        return false;
    }

    let local_part = parts[0];
    let domain_part = parts[1];

    if local_part.is_empty() || domain_part.is_empty() {
        return false; 
    }

    // Basic check for domain part: must contain at least one '.' and not start/end with '.'
    let domain_dots: Vec<&str> = domain_part.split('.').collect();
    if domain_dots.len() < 2 || domain_part.starts_with('.') || domain_part.ends_with('.') {
        return false;
    }

    // Ensure no spaces in the email address
    if email.contains(' ') {
        return false;
    }

    // More detailed checks for valid characters in local and domain parts would be needed
    // based on RFCs for a robust validation. This is a very simplified example.
    true
}


#[cfg(test)]
mod tests {
    use crate::domain::Contact;

    use super::*;

    #[test]
    fn test_validate_name() {
        assert!(validate_name("Alice"));
        assert!(!validate_name("123Bob"));
    }

    #[test]
    fn test_contact_from_line_ok() {
        let line = "Alice,12345,alice@email.com";
        let contact = Contact::from_line(line).unwrap();
        assert_eq!(contact.name, "Alice");
        assert_eq!(contact.phone, "12345");
        assert_eq!(contact.email, "alice@email.com");
    }

    #[test]
    fn test_validate_phone() {
        assert!(validate_phone("08123456789"));
        assert!(!validate_phone("1234"));
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("user@example.com"));
        assert!(!validate_email("invalid-email"));
    }
}