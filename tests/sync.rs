use chrono::Utc;
use rusty_rolodex::domain::Contact;
use rusty_rolodex::prelude::AppError;
use std::collections::HashSet;
// use serde_json::json;

fn make_contact(name: &str, email: &str, phone: &str) -> Contact {
    Contact {
        name: name.to_string(),
        email: email.to_string(),
        phone: phone.to_string(),
        tags: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[test]
fn test_merge_keep_policy() -> Result<(), AppError> {
    // Create existing contact
    let mut existing_contacts = vec![make_contact("Alice", "alice@example.com", "123")];

    let mut existing_keys: HashSet<(String, String)> = existing_contacts
        .iter()
        .map(|c| (c.name.clone(), c.email.clone()))
        .collect();

    // Incoming contact (duplicate)
    let imported_contacts = vec![
        make_contact("Alice", "alice@example.com", "999"),
        make_contact("Bob", "bob@example.com", "555"),
    ];

    for contact in imported_contacts {
        let key = (contact.name.clone(), contact.email.clone());

        if existing_keys.contains(&key) {
            continue;
        } else {
            existing_keys.insert(key);
            existing_contacts.push(contact);
        }
    }

    let merged = existing_contacts.clone();
    println!("Keep policy: {:?}", merged);
    // Should keep the original Alice (123), and add Bob
    assert_eq!(merged.len(), 2);
    assert!(merged.iter().any(|c| c.name == "Alice" && c.phone == "123"));
    assert!(merged.iter().any(|c| c.name == "Bob"));
    Ok(())
}

#[test]
fn test_merge_overwrite_policy() -> Result<(), AppError> {
    let mut existing_contacts = vec![make_contact("Alice", "alice@example.com", "123")];

    let imported_contacts = vec![make_contact("Alice", "alice@example.com", "999")];

    for contact in imported_contacts {
        let key = (contact.name.clone(), contact.email.clone());

        if let Some(pos) = existing_contacts
            .iter()
            .position(|c| c.name == key.0 && c.email == key.1)
        {
            existing_contacts[pos] = contact;
        } else {
            existing_contacts.push(contact);
        }
    }

    let merged = existing_contacts.clone();
    println!("Overwrite policy: {:?}", merged);

    // Should overwrite Alice’s phone to 999
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].phone, "999");
    Ok(())
}

#[test]
fn test_merge_duplicate_policy() -> Result<(), AppError> {
    let mut existing_contacts = vec![make_contact("Alice", "alice@example.com", "123")];

    let mut existing_keys: HashSet<(String, String)> = existing_contacts
        .iter()
        .map(|c| (c.name.clone(), c.email.clone()))
        .collect();

    let imported_contacts = vec![make_contact("Alice", "alice@example.com", "999")];

    for mut contact in imported_contacts {
        let key = (contact.name.clone(), contact.email.clone());

        if existing_keys.contains(&key) {
            contact.name = format!("{} (dup)", contact.name);
        }
        existing_keys.insert((contact.name.clone(), contact.email.clone()));
        existing_contacts.push(contact);
    }

    let merged = existing_contacts.clone();
    println!("Duplicate policy: {:?}", merged);
    // Should contain both entries: original and duplicate (renamed)
    assert_eq!(merged.len(), 2);
    assert!(merged.iter().any(|c| c.name.contains("(dup)")));
    Ok(())
}
