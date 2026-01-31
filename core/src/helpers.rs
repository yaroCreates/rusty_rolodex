use std::collections::HashSet;

use chrono::Utc;

use crate::domain::Contact;

pub fn merge_contact_data(local: &Contact, imported: &Contact) -> Contact {
    let mut merged = local.clone();
    
    // Use the more complete name
    if imported.name.len() > local.name.len() {
        merged.name = imported.name.clone();
    }
    
    // Use non-empty email, prefer the one from newer contact
    if !imported.email.is_empty() {
        if local.email.is_empty() || imported.updated_at > local.updated_at {
            merged.email = imported.email.clone();
        }
    }
    
    // Merge phone numbers (unique)
    let mut all_phones: HashSet<String> = local.phone.iter().cloned().collect();
    all_phones.extend(imported.phone.iter().cloned());
    merged.phone = all_phones.into_iter().collect();
    
    // Merge tags (unique)
    let mut all_tags: HashSet<String> = local.tags.iter().cloned().collect();
    all_tags.extend(imported.tags.iter().cloned());
    merged.tags = all_tags.into_iter().collect();
    
    // Use earlier created_at
    if imported.created_at < local.created_at {
        merged.created_at = imported.created_at;
    }
    
    // Use latest updated_at
    merged.updated_at = Utc::now();
    
    merged
}

pub fn completeness_score(contact: &Contact) -> usize {
    let mut score = 0;
    
    // Check name completeness
    if !contact.name.is_empty() {
        score += 1;
    }
    // if contact.name.contains(' ') {
    //     score += 1; // Has full name
    // }
    
    // Check email
    if !contact.email.is_empty() {
        score += 2;
    }
    
    // Phone numbers
    score += contact.phone.len() * 2;
    
    // Tags
    score += contact.tags.len();
    
    score
}