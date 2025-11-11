use chrono::Utc;
use rusty_rolodex::domain::Contact;
use rusty_rolodex::prelude::AppError;
use std::collections::HashSet;
// use serde_json::json;

fn make_contact(name: &str, email: &str, phone: &str) -> Contact {
    Contact {
        name: name.to_string(),
        email: email.to_string(),
        phone: vec![phone.to_string()],
        tags: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[test]
fn test_merge_keep_policy() -> Result<(), AppError> {
    // Create existing contact
    let mut existing_contacts = vec![make_contact("Alice", "alice@example.com", "123")];

    let mut existing_keys: HashSet<(String, Vec<String>)> = existing_contacts
        .iter()
        .map(|c| (c.name.clone(), c.phone.clone()))
        .collect();

    // Incoming contact (duplicate)
    let imported_contacts = vec![
        make_contact("Alice", "alice@example.com", "123"),
        make_contact("Bob", "bob@example.com", "555"),
    ];

    for contact in imported_contacts {
        let key = (contact.name.clone(), contact.phone.clone());

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
    assert!(
        merged
            .iter()
            .any(|c| c.name == "Alice" && c.phone.contains(&"123".to_string()))
    );
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
    assert_eq!(merged[0].phone, ["999"]);
    Ok(())
}

#[test]
fn test_merge_duplicate_policy() -> Result<(), AppError> {
    let mut existing_contacts = vec![make_contact("Alice", "alice@example.com", "123")];

    let mut existing_keys: HashSet<(String, Vec<String>)> = existing_contacts
        .iter()
        .map(|c| (c.name.clone(), c.phone.clone()))
        .collect();

    //duplicate contact
    let imported_contacts = vec![make_contact("Alice", "alice@example.com", "123")];

    for contact in imported_contacts {
        let key = (contact.name.clone(), contact.phone.clone());

        let imported_phone_set: HashSet<_> = contact.phone.iter().collect();

        for existing_contact in &mut existing_contacts {
            if existing_contact.name == contact.name
                && existing_contact
                    .phone
                    .iter()
                    .any(|p| imported_phone_set.contains(p))
            {
                existing_contact.phone.push(contact.phone.join(", "));

                if let Some(mut key) = existing_keys.take(&key) {
                    key.1.push(contact.phone.join(", "));
                    println!("{:?}", contact.phone);
                }
            }
        }
    }

    let merged = existing_contacts.clone();
    println!("Duplicate policy: {:?}", merged);
    // Should combine both phone entries
    assert_eq!(merged.len(), 1);
    assert!(merged.iter().any(|c| c.phone.len() == 2));
    Ok(())
}
