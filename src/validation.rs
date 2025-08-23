use regex::Regex;

pub fn validate_name(name: &str) -> bool {
    !name.trim().is_empty() && name.chars().all(|c| c.is_alphabetic() || c.is_whitespace())
}

pub fn validate_phone(phone: &str) -> bool {
    let re = Regex::new(r"^\d{10,}$").unwrap();
    re.is_match(phone)
}

pub fn validate_email(email: &str) -> bool {
    let re = Regex::new(r"^[\w\.-]+@[\w\.-]+\.\w+$").unwrap();
    re.is_match(email)
}
