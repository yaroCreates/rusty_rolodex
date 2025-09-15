use regex::Regex;

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
